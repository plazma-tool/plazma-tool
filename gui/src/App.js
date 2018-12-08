import React, { Component } from 'react';
import { render } from 'react-dom';
import MonacoEditor from 'react-monaco-editor';
import logo from './logo.svg';
import './App.css';

class PlasmaMonaco extends React.Component {
    constructor(props) {
        super(props);
        this.state = {
            code: '// hey monaco',
        };
    }

    editorDidMount(editor, monaco) {
        console.log('yay monaco', editor);
        editor.focus();
    }

    onChange(newValue, e) {
        console.log('onChange', newValue, e);
    }

    render() {
        const code = this.state.code;
        const options = {
            //selectOnLineNumbers: true,
        };

        return (
            <MonacoEditor
              width="800"
              height="600"
              language="javascript"
              theme="vs-dark"
              value={code}
              options={options}
              onChange={this.onChange}
              editorDidMount={this.editorDidMount}
            />
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
