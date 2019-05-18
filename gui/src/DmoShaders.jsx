// @flow
import React from 'react';
import { Panel, PanelBlock, PanelIcon, PanelHeading, Columns, Column } from 'bloomer';
import { CurrentPage } from './Helpers';
import type { DmoData } from './Helpers';

import MonacoEditor from 'react-monaco-editor';
import { ColorPickerColumns } from './PlazmaColorPicker';
import { PositionSlidersColumns } from './PlazmaPositionSliders';
import { SliderColumns } from './PlazmaSlider';
import { StatusBar } from './StatusBar';

import { GlslTokensProvider } from './Glsl/TokensProvider';
import { GlslCompletionProvider } from './Glsl/CompletionProvider';
import { GlslHoverProvider } from './Glsl/HoverProvider';
import { ThemeBase16DefaultDark } from './Glsl/ThemeBase16DefaultDark';

// TODO Use a collapsed and expanded state. Click on the menu label expands a
// tree. Selecting a shader opens it in the editor.

/*
function getShaderIndex(dmoData: DmoData, selectedPath: string) {
    if (dmoData === null) {
        return 0;
    }
    let idx = dmoData.context.index.shader_path_to_idx[selectedPath];
    if (idx === null) {
        console.log("Error: selectedPath not found in shaders");
        return 0;
    }
    let n = Number(idx);
    if (!isNaN(n)) {
        return n;
    } else {
        console.log("Error, index is not a number");
        return 0;
    }
}
*/

function pathBasename(path: string) {
    return path.replace(/.*\//, '')
}

type DSP_Props = {
    dmoData: DmoData,
    currentPage: number,
    currentIndex: number,
    onClickLift: () => void,
    onChangeLift: (idx: number) => void,
};

export class DmoShadersPanel extends React.Component<DSP_Props> {

    onChangeLocal = (currentIndex: number) => {
        this.props.onChangeLift(currentIndex);
    }

    render() {
        let paths: Array<[string, mixed]> = [];
        if (this.props.dmoData !== null) {
            paths = Object.entries(this.props.dmoData.context.index.shader_path_to_idx);
        };

        let pathLinks = paths.map((i: [string, mixed]) => {
            let path_full: string = i[0];
            let path_index: number = Number(i[1]);
            let is_active = false;
            let color = "";

            if (path_index === this.props.currentIndex) {
                is_active = true;
                color = "primary";
            }

            return (
                <PanelBlock
                    key={path_full}
                    isActive={is_active}
                    hasTextColor={color}
                    onClick={() => this.onChangeLocal(path_index)}
                    style={{ border: "none" }}
                >
                    <PanelIcon className="fa fa-code" />
                    {pathBasename(path_full)}
                </PanelBlock>
            );
        });

        let color = "";
        if (this.props.currentPage === CurrentPage.Shaders) {
            color = "primary";
        }

        return (
            <Panel onClick={this.props.onClickLift}>
                <PanelHeading hasTextColor={color}>Shaders</PanelHeading>
                {pathLinks}
            </Panel>
        );
    }
}

type SP_Props = {
    editorContent: string,
    onChange_PlazmaMonaco: (newValue: string, e: MessageEvent) => void,
    onChange_ColorPickerColumns: (newValue: string) => void,
    onChange_PositionSlidersColumns: (newValue: string) => void,
    onChange_SliderColumns: (newValue: string) => void,
};

export class ShadersPage extends React.Component<SP_Props> {
    render() {
        return(
            <div>
                <Columns>
                    <Column>
                        <PlazmaMonaco
                            editorContent={this.props.editorContent}
                            onChangeLift={this.props.onChange_PlazmaMonaco}
                        />
                    </Column>
                </Columns>
                <Columns>
                    <ColorPickerColumns
                        code={this.props.editorContent}
                        onChangeLift={this.props.onChange_ColorPickerColumns}
                    />
                    <PositionSlidersColumns
                        code={this.props.editorContent}
                        onChangeLift={this.props.onChange_PositionSlidersColumns}
                    />
                    <SliderColumns
                        code={this.props.editorContent}
                        onChangeLift={this.props.onChange_SliderColumns}
                    />
                </Columns>
            </div>
        );
    }
}

function UndoRedoButton(props) {
    return (
        <button onClick={props.onClick} disabled={props.disabled}>{props.label}</button>
    );
}

type PMT_Props = {
    editor: MonacoEditor,
    undoDisabled: bool,
    redoDisabled: bool,
};

class PlazmaMonacoToolbar extends React.Component<PMT_Props> {

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

type PM_Props = {
    editorContent: string,
    onChangeLift: (newValue: string, e: MessageEvent) => void,
};

type PM_State = {
    editor: MonacoEditor,
    modelVersions: {
        initialVersion: number,
        currentVersion: number,
        lastVersion: number,
    },
    undoDisabled: bool,
    redoDisabled: bool,
};

class PlazmaMonaco extends React.Component<PM_Props, PM_State> {
    constructor(props) {
        super(props);

        this.state = {
            editor: null,
            modelVersions: {
                initialVersion: 0,
                currentVersion: 0,
                lastVersion: 0,
            },
            undoDisabled: true,
            redoDisabled: true,
        };
    }

    editorDidMount = (editor, monaco) => {
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

    onChangeLocal = (newValue, e) => {
        this.props.onChangeLift(newValue, e);
        this.updateVersions();
    }

    onResize = () => {
        this.state.editor.layout({height: 0, width: 0});
        this.state.editor.layout();
    }

    // FIXME: redo is disabled before the last action is restored (last edit
    // can't be restored).

    updateVersions = () => {
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
        monaco.languages.setMonarchTokensProvider('glsl', GlslTokensProvider);
        monaco.languages.registerCompletionItemProvider('glsl', GlslCompletionProvider);
        monaco.languages.registerHoverProvider('glsl', GlslHoverProvider);
        monaco.editor.defineTheme('glsl-base16-default-dark', ThemeBase16DefaultDark);
    }

    render() {
        const options = {
            language: "glsl",
            theme: "glsl-base16-default-dark",
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
                theme="glsl-base16-default-dark"
                fontFamily="Iosevka Term Web"
                value={this.props.editorContent}
                options={options}
                onChange={this.onChangeLocal}
                editorWillMount={this.editorWillMount}
                editorDidMount={this.editorDidMount}
              />

              <StatusBar
                  editorContent={this.props.editorContent}
              />
            </div>
        );
    }
}

