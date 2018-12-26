import React, { Component } from 'react';
//import * as ReactDOM from 'react-dom';
import { Icon, Button, Container, Box, Menu, MenuLabel, MenuList, MenuLink, Columns, Column  } from 'bloomer';
import MonacoEditor from 'react-monaco-editor';
import { SketchPicker } from 'react-color';
import Slider, { Range } from 'rc-slider';
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

const default_shader = [
    '#version 430',
    '',
    'in vec2 texCoord;',
    'out vec4 out_color;',
    '',
    'layout(location = 0) uniform float iTime;',
    'layout(location = 1) uniform vec2 iResolution;',
    'layout(location = 3) uniform vec2 screenResolution;',
    '',
    '// --- tool ---',
    '',
    'void mainImage( out vec4 fragColor, in vec2 fragCoord )',
    '{',
    '  // Normalized pixel coordinates (from 0 to 1)',
    '  vec2 uv = fragCoord/iResolution.xy;',
    '',
    '  // Time varying pixel color',
    '  vec3 col = 0.5 + 0.5*cos(iTime+uv.xyx+vec3(0,2,4));',
    '',
    '  // Output to screen',
    '  fragColor = vec4(col,1.0);',
    '}',
    '',
    '// --- tool ---',
    '',
    'void main() {',
    '  vec4 col = vec4(0.0, 0.0, 0.0, 1.0);',
    '  mainImage(col, gl_FragCoord.xy);',
    '  out_color = col;',
    '}',
].join('\n');

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

    render() {
        const options = {
            //language: "glsl",
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
                language="plaintext"
                theme="vs-dark"
                value={this.props.editorContent}
                options={options}
                onChange={this.onChangeLocal}
                editorDidMount={this.editorDidMount}
              />
            </div>
        );
    }
}

class App extends Component {
    constructor(props) {
        super(props);

        this.state = {
            socket: null,
            dmo_data: null,
            editor_content: null,
            sentUpdateSinceChange: false,
        };

        this.updateTimerId = null;

        this.onEditorChange = this.onEditorChange.bind(this);
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
            let frag_src = d.context_data.quad_scenes[0].frag_src;
            this.setState({
                dmo_data: d,
                editor_content: frag_src,
            });
            this.setState({
                sentUpdateSinceChange: true,
            });
        }
    }

    onEditorChange(newValue, e) {
        if (this.state.dmo_data) {
            let d = this.state.dmo_data;
            d.context_data.quad_scenes[0].frag_src = newValue;

            this.setState({
                dmo_data: d,
                editor_content: newValue,
            });
        }
        this.setState({
            sentUpdateSinceChange: false,
        });
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
                    <MenuLabel>Textures</MenuLabel>
                    <MenuList>
                      <li><MenuLink>Medium RGBA Noise</MenuLink></li>
                      <li><MenuLink>Rock</MenuLink></li>
                      <li><MenuLink>Street</MenuLink></li>
                    </MenuList>
                    <MenuLabel>Shaders</MenuLabel>
                    <MenuList>
                      <li><MenuLink>background</MenuLink></li>
                      <li><MenuLink>text</MenuLink></li>
                      <li><MenuLink>raymarch</MenuLink></li>
                      <li><MenuLink>bloom</MenuLink></li>
                      <li><MenuLink>compositing</MenuLink></li>
                    </MenuList>
                  </Menu>
                  <div>
                    <Button isActive isColor='primary'>Variables</Button>
                  </div>
                  <div>
                    <Button>Samplers</Button>
                  </div>
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
                    <Column>
                      <SketchPicker/>
                    </Column>
                    <Column>
                      <SketchPicker/>
                    </Column>
                    <Column>
                      <Slider />
                      <Range />
                    </Column>
                  </Columns>
                </Column>
              </Columns>
            </div>
        );
    }
}

export default App;

