import React from 'react';
import { Columns, Column, Title, MenuList, MenuLink } from 'bloomer';
import { CurrentPage } from './Helpers';

import MonacoEditor from 'react-monaco-editor';
import { ColorPickerColumns } from './PlazmaColorPicker';
import { PositionSlidersColumns } from './PlazmaPositionSliders';
import { SliderColumns } from './PlazmaSlider';

// TODO Use a collapsed and expanded state. Click on the menu label expands a
// tree. Selecting a shader opens it in the editor.

/*
function getShaderIndex(dmoData, selectedPath) {
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

function pathBasename(path) {
    return path.replace(/.*\//, '')
}

// Requires props:
// - dmoData
// - currentIndex
// - currentPage
// - onChangeLift
// - onClickLift
export class DmoShadersMenu extends React.Component {
    constructor(props) {
        super(props);
        this.onChangeLocal = this.onChangeLocal.bind(this);
    }

    onChangeLocal(currentIndex) {
        this.props.onChangeLift(currentIndex);
    }

    render() {
        let paths = [];
        if (this.props.dmoData !== null) {
            paths = Object.entries(this.props.dmoData.context.index.shader_path_to_idx);
        };

        let pathLinks = paths.map((i) => {
            let path_full = i[0];
            let path_index = i[1];
            let link;

            if (path_index === this.props.currentIndex) {
                link = <MenuLink isActive>{pathBasename(path_full)}</MenuLink>
            } else {
                link = <MenuLink>{pathBasename(path_full)}</MenuLink>
            };
            return (
                <li
                    key={path_full}
                    onClick={() => this.onChangeLocal(path_index)}
                >
                    {link}
                </li>
            );
        });

        let color = "";
        if (this.props.currentPage === CurrentPage.Shaders) {
            color = "primary";
        }

        return (
            <div onClick={this.props.onClickLift}>
                <Title tag='h1' hasTextColor={color}>Shaders</Title>
                <MenuList>
                    {pathLinks}
                </MenuList>
            </div>
        );
    }
}

// Requires props:
// - editorContent
export class ShadersPage extends React.Component {
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

