import React, { Component } from 'react';
//import * as ReactDOM from 'react-dom';
import { Icon, Button, Menu, MenuLabel, MenuList, MenuLink, Columns, Column  } from 'bloomer';
import MonacoEditor from 'react-monaco-editor';
import { ColorPickerColumns } from './PlazmaColorPicker';
import { PositionSlidersColumns } from './PlazmaPositionSliders';
import { SliderColumns } from './PlazmaSlider';
import { DmoSettingsMenu } from './DmoSettings';
import { DmoContextMenu } from './DmoContext';
import { DmoTimelineMenu } from './DmoTimeline';
import './App.css';

//import logo from './logo.svg';

const PLAZMA_SERVER_PORT = 8080;

function UndoRedoButton(props) {
    return (
        <button onClick={props.onClick} disabled={props.disabled}>{props.label}</button>
    );
}

class PlazmaMonacoToolbar extends React.Component {

    undoAction(editor) {
        if (editor) {
            editor.trigger('aaaa', 'undo', 'aaaa');
            editor.focus();
        }
    }

    redoAction(editor) {
        if (editor) {
            editor.trigger('aaaa', 'redo', 'aaaa');
            editor.focus();
        }
    }

    render() {
        return (
            <div className="toolbar">
              <UndoRedoButton
                onClick={() => this.undoAction(this.props.editor)}
                disabled={this.props.undoDisabled}
                label="Undo"
              />

              <UndoRedoButton
                onClick={() => this.redoAction(this.props.editor)}
                disabled={this.props.redoDisabled}
                label="Redo"
              />
            </div>
        );
    }
}

// Requires props:
// - editorContent
// - onChangeLift
class PlazmaMonaco extends React.Component {
    constructor(props) {
        super(props);

        this.state = {
            editor: null,
            modelVersions: null,
            undoDisabled: true,
            redoDisabled: true,
        };

        this.editorDidMount = this.editorDidMount.bind(this);
        this.onResize = this.onResize.bind(this);
        this.onChangeLocal = this.onChangeLocal.bind(this);
        this.updateVersions = this.updateVersions.bind(this);
    }

    editorDidMount(editor, monaco) {
        editor.getModel().setValue(this.props.editorContent);

        let id = editor.getModel().getAlternativeVersionId();
        let modelVersions = {
            initialVersion: id,
            currentVersion: id,
            lastVersion: id,
        };

        // not using automaticLayout on the editor, b/c it adds a 100ms interval listener.
        // https://github.com/Microsoft/monaco-editor/issues/28

        window.addEventListener('resize', this.onResize);

        this.setState({
            editor: editor,
            modelVersions: modelVersions,
        });

        editor.focus();
        editor.setPosition({ lineNumber: 1, column: 1 });
    }

    onChangeLocal(newValue, e) {
        this.props.onChangeLift(newValue, e);
        this.updateVersions();
    }

    onResize() {
        this.state.editor.layout({height: 0, width: 0});
        this.state.editor.layout();
    }

    // FIXME: redo is disabled before the last action is restored (last edit
    // can't be restored).

    updateVersions() {
        if (!this.state.modelVersions) {
            return;
        }
        let mv = this.state.modelVersions;
        let undoDisabled = this.state.undoDisabled;
        let redoDisabled = this.state.redoDisabled;

        const versionId = this.state.editor.getModel().getAlternativeVersionId();

        // undoing
        if (versionId < mv.currentVersion) {
            redoDisabled = false;
            // no more undo possible
            if (versionId === mv.initialVersion) {
                undoDisabled = true;
            }
        } else {
            // redoing
            if (versionId <= mv.lastVersion) {
                // redoing the last change
                if (versionId === mv.lastVersion) {
                    redoDisabled = true;
                }
            } else {
                // adding new change, disable redo when adding new changes
                redoDisabled = true;
                if (mv.currentVersion > mv.lastVersion) {
                    mv.lastVersion = mv.currentVersion;
                }
            }
            undoDisabled = false;
        }
        mv.currentVersion = versionId;

        this.setState({
            modelVersions: mv,
            undoDisabled: undoDisabled,
            redoDisabled: redoDisabled,
        });
    }

    editorWillMount(monaco) {
        monaco.languages.register({ id: 'glsl' });
        monaco.languages.setMonarchTokensProvider('glsl', {
            comments: {
                "lineComment": "//",
                "blockComment": [ "/*", "*/" ]
            },
            brackets: [
                ["{", "}", "delimiter.curly"],
                //["[", "]", "delimiter.bracket"],
                //["(", ")", "delimiter.round"],
            ],
            tokenizer: {
                root: [],
            },
        });
    }

    render() {
        const options = {
            language: "glsl",
            lineNumbers: "on",
            roundedSelection: false,
            scrollBeyondLastLine: true,
        };

        return (
            <div>
              <PlazmaMonacoToolbar
                editor={this.state.editor}
                undoDisabled={this.state.undoDisabled}
                redoDisabled={this.state.redoDisabled}
              />

              <MonacoEditor
                //width="800"
                height="600"
                language="glsl"
                theme="vs-dark"
                value={this.props.editorContent}
                options={options}
                onChange={this.onChangeLocal}
                editorWillMount={this.editorWillMount}
                editorDidMount={this.editorDidMount}
              />
            </div>
        );
    }
}

class App extends Component {
    constructor(props) {
        super(props);

        // TODO add a new attribute to select what is displayed in the main
        // panel.

        // NOTE No 0 index to avoid == problems.
        const CurrentPage = {
            Settings: 1,
            ContextShader: 2,
            Timeline: 3,
        };

        this.state = {
            socket: null,
            dmo_data: null,
            editor_content: null,
            current_page: CurrentPage.ContextShader,
            current_shader_index: null,
            sentUpdateSinceChange: false,
        };

        this.updateTimerId = null;

        this.sendUpdatedContent = this.sendUpdatedContent.bind(this);
        this.onEditorChange = this.onEditorChange.bind(this);
        this.onContextMenuChange = this.onContextMenuChange.bind(this);
        this.onColorPickerChange = this.onColorPickerChange.bind(this);
        this.onPositionSlidersChange = this.onPositionSlidersChange.bind(this);
        this.handleSocketOpen = this.handleSocketOpen.bind(this);
        this.handleSocketMessage = this.handleSocketMessage.bind(this);
        this.sendDmoData = this.sendDmoData.bind(this);
    }

    componentDidMount() {
        const socket = new WebSocket('ws://localhost:' + PLAZMA_SERVER_PORT + '/ws/');

        socket.addEventListener('open', this.handleSocketOpen);
        socket.addEventListener('message', this.handleSocketMessage);

        this.updateTimerId = window.setInterval(this.sendDmoData, 1000);

        this.setState({
            socket: socket,
        });
    }

    componentWillUnmount() {
        window.clearInterval(this.updateTimerId);
    }

    handleSocketOpen(event) {
        // Request DmoData from server.
        let msg = {
            data_type: 'FetchDmo',
            data: '',
        };
        this.state.socket.send(JSON.stringify(msg));
        this.setState({
            sentUpdateSinceChange: true,
        });
    }

    handleSocketMessage(event) {
        var msg = JSON.parse(event.data);
        if (msg.data_type === 'SetDmo') {
            let d = JSON.parse(msg.data);

            let idx = this.state.current_shader_index;
            let frag_src = d.context.shader_sources[idx];
            this.setState({
                dmo_data: d,
                editor_content: frag_src,
            });
            this.setState({
                sentUpdateSinceChange: true,
            });
        }
    }

    sendUpdatedContent(newValue) {
        if (this.state.dmo_data) {
            let d = this.state.dmo_data;
            let idx = this.state.current_shader_index;
            d.context.shader_sources[idx] = newValue;

            this.setState({
                dmo_data: d,
                editor_content: newValue,
            });
        }
        this.setState({
            sentUpdateSinceChange: false,
        });
    }

    onContextMenuChange(idx) {
        this.setState({
            current_shader_index: idx,
            editor_content: this.state.dmo_data.context.shader_sources[idx],
        });
    }

    onEditorChange(newValue, e) {
        this.sendUpdatedContent(newValue);
    }

    onColorPickerChange(newValue) {
        this.sendUpdatedContent(newValue);
    }

    onPositionSlidersChange(newValue) {
        this.sendUpdatedContent(newValue);
    }

    sendDmoData() {
        if (this.state.sentUpdateSinceChange) {
            return;
        } else if (this.state.socket) {
            let msg = {
                data_type: 'SetDmo',
                data: JSON.stringify(this.state.dmo_data),
            };
            this.state.socket.send(JSON.stringify(msg));
            this.setState({
                sentUpdateSinceChange: true,
            });
        }
    }

    render() {
        return (
            <div className="App">
              <Columns>
                <Column isSize={{default: 1}}>
                  <Menu>
                    <DmoSettingsMenu/>
                    <DmoContextMenu
                        dmoData={this.state.dmo_data}
                        currentIndex={this.state.current_shader_index}
                        onChangeLift={this.onContextMenuChange}
                    />
                    <DmoTimelineMenu/>
                  </Menu>

                  {/*
                  <div>
                    <Button>
                      <Icon className="fas fa-fast-backward fa-lg" />
                    </Button>
                    <Button isColor='success' isOutlined>
                      <Icon className="fas fa-play fa-lg" />
                    </Button>
                    <Button>
                      <Icon className="fas fa-fast-forward fa-lg" />
                    </Button>
                  </div>
                  */}
                </Column>
                <Column>
                  <Columns>
                    <Column>
                      <PlazmaMonaco
                        editorContent={this.state.editor_content}
                        onChangeLift={this.onEditorChange}
                      />
                    </Column>
                  </Columns>
                  <Columns>

                    <ColorPickerColumns
                      code={this.state.editor_content}
                      onChangeLift={this.onColorPickerChange}
                    />

                    <PositionSlidersColumns
                      code={this.state.editor_content}
                      onChangeLift={this.onPositionSlidersChange}
                    />

                    <SliderColumns
                      code={this.state.editor_content}
                      onChangeLift={this.onColorPickerChange}
                    />

                  </Columns>
                </Column>
              </Columns>
            </div>
        );
    }
}

export default App;

