import React, { Component } from 'react';
//import { render } from 'react-dom';
import MonacoEditor from 'react-monaco-editor';
//import logo from './logo.svg';
import './App.css';

function UndoRedoButton(props) {
    return (
        <button onClick={props.onClick} disabled={props.disabled}>{props.label}</button>
    );
}

class PlasmaMonacoToolbar extends React.Component {

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


class PlasmaMonaco extends React.Component {
    constructor(props) {
        super(props);

        this.state = {
            editor: null,
            socket: null,
            modelVersions: null,
            undoDisabled: true,
            redoDisabled: true,
        };

        this.initialCode = default_shader;
        this.sentUpdateSinceChange = false;
        this.updateTimerId = null;

        this.editorDidMount = this.editorDidMount.bind(this);
        this.onChange = this.onChange.bind(this);
        this.updateVersions = this.updateVersions.bind(this);
        this.sendSetFragmentShader = this.sendSetFragmentShader.bind(this);
        this.handleSocketOpen = this.handleSocketOpen.bind(this);
        this.handleSocketMessage = this.handleSocketMessage.bind(this);
    }

    handleSocketOpen(event) {
        var msg = {
            data_type: 'SetFragmentShader',
            data: this.initialCode,
        };
        this.state.socket.send(JSON.stringify(msg));
    }

    handleSocketMessage(event) {
        var msg = JSON.parse(event.data);
        if (msg.data_type === 'SetFragmentShader') {
            this.state.editor.getModel().setValue(JSON.parse(msg.data));
            this.sentUpdateSinceChange = true;
        }
    }

    editorDidMount(editor, monaco) {
        editor.getModel().setValue(this.initialCode);

        let id = editor.getModel().getAlternativeVersionId();
        let modelVersions = {
            initialVersion: id,
            currentVersion: id,
            lastVersion: id,
        };

        this.updateTimerId = window.setInterval(this.sendSetFragmentShader, 1000);

        const socket = new WebSocket('ws://localhost:8080/ws/');

        socket.addEventListener('open', this.handleSocketOpen);
        socket.addEventListener('message', this.handleSocketMessage);

        this.setState({
            editor: editor,
            socket: socket,
            modelVersions: modelVersions,
        });

        editor.focus();
        editor.setPosition({ lineNumber: 1, column: 1 });
    }

    onChange(newValue, e) {
        this.sentUpdateSinceChange = false;
        this.updateVersions();
    }

    sendSetFragmentShader() {
        if (this.sentUpdateSinceChange) {
            return;
        } else {
            var msg = {
                data_type: 'SetFragmentShader',
                data: this.state.editor.getModel().getValue(),
            };
            this.state.socket.send(JSON.stringify(msg));
            this.sentUpdateSinceChange = true;
        }
    }

    componentWillUnmount() {
        window.clearInterval(this.updateTimerId);
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
              <PlasmaMonacoToolbar
                editor={this.state.editor}
                undoDisabled={this.state.undoDisabled}
                redoDisabled={this.state.redoDisabled}
              />

              <MonacoEditor
                width="800"
                height="600"
                language="plaintext"
                theme="vs-dark"
                //value={code}
                options={options}
                onChange={this.onChange}
                editorDidMount={this.editorDidMount}
              />
            </div>
        );
    }
}

class App extends Component {
  render() {
    return (
      <div className="App">
        <PlasmaMonaco />
      </div>
    );
  }
}

export default App;

