// @flow
import React from 'react';
import MonacoEditor from 'react-monaco-editor';

import { GlslTokensProvider } from './Glsl/TokensProvider';
import { ThemeBase16DefaultDark } from './Glsl/ThemeBase16DefaultDark';

type SB_Props = {
    editorContent: string,
};

type SB_State = {
    editor: MonacoEditor,
};

export class StatusBar extends React.Component<SB_Props, SB_State> {
    constructor(props) {
        super(props);

        this.state = {
            editor: null,
        };
    }

    editorDidMount = (editor, monaco) => {
        window.addEventListener('resize', this.onResize);
        this.setState({
            editor: editor,
        });
        editor.setPosition({ lineNumber: 1, column: 1 });
    }

    onResize = () => {
        this.state.editor.layout({height: 0, width: 0});
        this.state.editor.layout();
    }

    editorWillMount(monaco) {
        monaco.languages.register({ id: 'glsl' });
        monaco.languages.setMonarchTokensProvider('glsl', GlslTokensProvider);
        monaco.editor.defineTheme('glsl-base16-default-dark', ThemeBase16DefaultDark);
    }

    render() {
        const options = {
            language: "glsl",
            theme: "glsl-base16-default-dark",
            lineNumbers: "off",
            readOnly: true,
            scrollbar: { vertical: 'hidden', horizontal: 'hidden' },
            hover: { enabled: false },
            minimap: { enabled: false },
        };

        return (
              <MonacoEditor
                height="60"
                language="glsl"
                theme="glsl-base16-default-dark"
                fontFamily="Iosevka Term Web"
                value={this.props.editorContent}
                options={options}
                editorWillMount={this.editorWillMount}
                editorDidMount={this.editorDidMount}
              />
        );
    }
}
