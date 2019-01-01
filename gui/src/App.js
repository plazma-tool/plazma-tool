import React, { Component } from 'react';
//import * as ReactDOM from 'react-dom';
import { Icon, Button, Menu, MenuLabel, MenuList, MenuLink, Columns, Column  } from 'bloomer';
import MonacoEditor from 'react-monaco-editor';
import { SketchPicker } from 'react-color';
import Slider from 'rc-slider';
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

function getColorValuesFromCode(code) {
    let values = [];
    if (code === null) {
        return values;
    }

    let re_color = /vec3 +([^ ]+) *= *vec3\(([^\)]+)\); *\/\/ *!! color *$/gm;

    let m = null;
    while ((m = re_color.exec(code)) !== null) {
        let name = m[1].trim();
        let rgb = m[2].trim();
        let vec = [];

        let mm = rgb.match(/([0-9\.]+)/g);
        if (mm !== null) {
            mm.forEach((i) => {
                let n = Number(i);
                if (!isNaN(n)) {
                    let nn = Math.floor(n * 255);
                    vec.push(nn);
                }
            });
            if (vec.length === 3) {
                values.push({
                    name: name,
                    rgba: {
                        r: vec[0],
                        g: vec[1],
                        b: vec[2],
                        a: 1.0,
                    },
                });
            }
        }
    }

    return values;
}

function getSliderValuesFromCode(code) {
    let values = [];
    if (code === null) {
        return values;
    }

    let re_slider = /float +([^ ]+) *= *([0-9\.]+); *\/\/ *!! slider *$/gm;

    let m = null;
    while ((m = re_slider.exec(code)) !== null) {
        values.push({
            name: m[1].trim(),
            value: Math.floor(Number(m[2].trim()) * 1000),
        });
    }

    return values;
}

function numToStrPad(x) {
    let s = x.toFixed(3).toString();
    if (s.indexOf('.') === -1) {
        return s + '.000';
    } else {
        return s.padEnd(5, '0');
    }
}

function rgbaToVec3(rgba) {
    let vec = [ rgba.r, rgba.g, rgba.b ].map((i) => {
        return numToStrPad(Number((i / 255)));
    });
    return 'vec3(' + vec[0] + ', ' + vec[1] + ', ' + vec[2] + ')';
}

function replaceColorValueInCode(newColorValue, code) {
    const c = newColorValue;
    let re_color = new RegExp('(vec3 +' + c.name + ' *= *)vec3\\([^\\)]+\\)(; *\\/\\/ *!! color *$)', 'gm');
    let newCodeValue = code.replace(re_color, '$1' + rgbaToVec3(c.rgba) + '$2');
    return newCodeValue;
}

function replaceSliderValueInCode(newSliderValue, code) {
    const x = newSliderValue;
    let re_slider = new RegExp('(float ' + x.name + ' *= *)[0-9\\.]+(; *\\/\\/ *!! slider *$)', 'gm');
    let newCodeValue = code.replace(re_slider, '$1' + numToStrPad(x.value / 1000) + '$2');
    return newCodeValue;
}

// Requires props:
// - color: { name: "name", rgba: { r: 0, g: 0, b: 0, a: 0 } }
// - onChangeLift
class PlazmaColorPicker extends React.Component {
    constructor(props) {
        super(props);
        this.onChangeLocal = this.onChangeLocal.bind(this);
    }

    onChangeLocal(color, event) {
        let c = this.props.color;
        let newColorValue = {
            name: c.name,
            rgba: color.rgb,
        };
        this.props.onChangeLift(newColorValue);
    }

    render() {
        let c = this.props.color;
        return (
            <div className="is-one-quarter">
              <span>{c.name}</span>
              <SketchPicker
                color={c.rgba}
                onChange={this.onChangeLocal}
              />
            </div>
        );
    }
}

// Requires props:
// - sliderValue: { name: "name", value: 0.0 }
// - onChangeLift
class PlazmaSlider extends React.Component {
    constructor(props) {
        super(props);
        this.onChangeLocal = this.onChangeLocal.bind(this);
    }

    onChangeLocal(x) {
        let newValue = {
            name: this.props.sliderValue.name,
            value: x,
        };
        this.props.onChangeLift(newValue);
    }

    render() {
        return (
            <div className="is-half">
              <span>{this.props.sliderValue.name}</span>
              <Slider
                value={this.props.sliderValue.value}
                step={1}
                min={0}
                max={1000}
                onChange={this.onChangeLocal}
              />
            </div>
        );
    }
}

// Requires props:
// - code
// - onChangeLift
class ColorPickerColumns extends React.Component {
    constructor(props) {
        super(props);
        this.onChangeLocal = this.onChangeLocal.bind(this);
    }

    onChangeLocal(newColorValue) {
        let newCodeValue = replaceColorValueInCode(newColorValue, this.props.code);
        this.props.onChangeLift(newCodeValue);
    }

    render() {
        let values = getColorValuesFromCode(this.props.code);
        let pickers = values.map((color, idx) => {
            return (
                <PlazmaColorPicker
                  key={color.name + idx}
                  color={color}
                  onChangeLift={this.onChangeLocal}
                />
            );
        });
        return (
            <Column>
              <Columns>
                {pickers}
              </Columns>
            </Column>
        );
    }
}

// Requires props:
// - code
// - onChangeLift
class SliderColumns extends React.Component {
    constructor(props) {
        super(props);
        this.onChangeLocal = this.onChangeLocal.bind(this);
    }

    onChangeLocal(newValue) {
        let newCodeValue = replaceSliderValueInCode(newValue, this.props.code);
        this.props.onChangeLift(newCodeValue);
    }

    render() {
        let values = getSliderValuesFromCode(this.props.code);
        let sliders = values.map((value, idx) => {
            return (
                <PlazmaSlider
                  key={value.name + idx}
                  sliderValue={value}
                  onChangeLift={this.onChangeLocal}
                />
            );
        });
        return (
            <Column>
              {sliders}
            </Column>
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

        this.state = {
            socket: null,
            dmo_data: null,
            editor_content: null,
            sentUpdateSinceChange: false,
        };

        this.updateTimerId = null;

        this.sendUpdatedContent = this.sendUpdatedContent.bind(this);
        this.onEditorChange = this.onEditorChange.bind(this);
        this.onColorPickerChange = this.onColorPickerChange.bind(this);
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
            let frag_src = d.context.quad_scenes[0].frag_src;
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
            d.context.quad_scenes[0].frag_src = newValue;

            this.setState({
                dmo_data: d,
                editor_content: newValue,
            });
        }
        this.setState({
            sentUpdateSinceChange: false,
        });
    }

    onEditorChange(newValue, e) {
        this.sendUpdatedContent(newValue);
    }

    onColorPickerChange(newValue) {
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

                    <ColorPickerColumns
                      code={this.state.editor_content}
                      onChangeLift={this.onColorPickerChange}
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

