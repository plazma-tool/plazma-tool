// @flow
import React from 'react';
import { Panel, PanelBlock, PanelIcon, PanelHeading, Columns, Column, Level, LevelItem, LevelLeft } from 'bloomer';
import { CurrentPage, EditorsLayout, parseShaderErrorText, pathBasename } from './Helpers';
import type { DmoData, Shader, ShaderEditors, ShaderErrorMessage } from './Helpers';

import MonacoEditor from 'react-monaco-editor';
import { ColorPickerColumns } from './PlazmaColorPicker';
import { PositionSlidersColumns } from './PlazmaPositionSliders';
import { SliderColumns } from './PlazmaSlider';
import { MessageArea } from './MessageArea';

import { GlslTokensProvider } from './Glsl/TokensProvider';
import { GlslCompletionProvider } from './Glsl/CompletionProvider';
import { GlslHoverProvider } from './Glsl/HoverProvider';
import { ThemeBase16DefaultDark } from './Glsl/ThemeBase16DefaultDark';

type DSP_Props = {
    shaders: Shader[],
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
        let pathLinks = this.props.shaders
        // map first to preserve the index order
            .map((shader: Shader, shader_idx: number) => {
                let is_active = false;
                let label_color = "";
                let icon_color = "";

                if (shader.error_data !== null && typeof shader.error_data !== 'undefined') {
                    label_color = "error";
                    icon_color = "error";
                }

                if (shader_idx === this.props.currentIndex) {
                    is_active = true;
                    label_color = "primary";
                }

                return ({
                    file_path: shader.file_path,
                    item: <PanelBlock
                        key={shader.file_path + label_color + icon_color}
                        isActive={is_active}
                        hasTextColor={label_color}
                        onClick={() => this.onChangeLocal(shader_idx)}
                        style={{ border: "none" }}
                    >
                        <PanelIcon className="fa fa-code" hasTextColor={icon_color} />
                        {shader.file_path}
                    </PanelBlock>
                });
            })
        // filter out the builtin shaders
            .filter((i) => !i.file_path.startsWith('data_builtin_'))
            .map((i) => i.item);

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
    shaders: Shader[],
    shaderEditors: ShaderEditors,
    onChange_PlazmaMonaco: (newShader: Shader) => void,
    onFocus_PlazmaMonaco: (editorIdx: number) => void,
    onBlur_PlazmaMonaco: (editorIdx: number, viewState: {}) => void,
    onChange_ColorPickerColumns: (newShader: Shader) => void,
    onChange_PositionSlidersColumns: (newShader: Shader) => void,
    onChange_SliderColumns: (newShader: Shader) => void,
    monacoDidInit: bool,
    onMonacoDidInit: () => void,
};

export class ShadersPage extends React.Component<SP_Props> {

    createEditorsWithHeights = (heights?: number[]): MonacoEditor[] => {
        let a = this.props.shaderEditors.editors.map((i, idx) => {
            let h = Math.floor(this.props.shaderEditors.full_height / 2);
            if (heights !== null && typeof heights !== 'undefined') {
                h = (heights[idx] !== null && typeof heights[idx] !== 'undefined') ? heights[idx] : h;
            }
            return (
                <PlazmaMonaco
                    key={'PlazmaMonaco_'+idx}
                    editorIdx={idx}
                    editorHeight={h}
                    shaderEditors={this.props.shaderEditors}
                    shader={this.props.shaders[i.source_idx]}
                    onChangeLift={this.props.onChange_PlazmaMonaco}
                    onFocusLift={this.props.onFocus_PlazmaMonaco}
                    onBlurLift={this.props.onBlur_PlazmaMonaco}
                    monacoDidInit={this.props.monacoDidInit}
                    onMonacoDidInit={this.props.onMonacoDidInit}
                />
            );
        });
        return a;
    }

    render() {
        let current_src_idx = this.props.shaderEditors.editors[this.props.shaderEditors.current_editor_idx].source_idx;
        let current_shader = this.props.shaders[current_src_idx];

        let full = this.props.shaderEditors.full_height;
        let half = Math.floor((full - 24) / 2); // 24px = height of the <ShaderStatusBar>
        let editors = [];
        let editor_columns = <div></div>;

        switch (this.props.shaderEditors.layout) {
            case EditorsLayout.OneMax:
                editors = this.createEditorsWithHeights([full]);
                editor_columns = <Column> {editors[0]} </Column>;
                break;

            case EditorsLayout.TwoVertical:
                editors = this.createEditorsWithHeights([full, full]);
                editor_columns = [
                    <Column key='Col_1'> {editors[0]} </Column>,
                    <Column key='Col_2'> {editors[1]} </Column>
                ];
                break;

            case EditorsLayout.TwoHorizontal:
                editors = this.createEditorsWithHeights([half, half]);
                editor_columns =
                    <Column>
                        {editors[0]}
                        {editors[1]}
                    </Column>;
                break;

            case EditorsLayout.ThreeMainLeft:
                editors = this.createEditorsWithHeights([full, half, half]);
                editor_columns = [
                    <Column key='Col_1'> {editors[0]} </Column>,
                    <Column key='Col_2'>
                        {editors[1]}
                        {editors[2]}
                    </Column>
                ];
                break;

            case EditorsLayout.ThreeMainRight:
                editors = this.createEditorsWithHeights([full, half, half]);
                editor_columns = [
                    <Column key='Col_1'>
                        {editors[1]}
                        {editors[2]}
                    </Column>,
                    <Column key='Col_2'> {editors[0]} </Column>,
                ];
                break;

            case EditorsLayout.ThreeMainTop:
                editors = this.createEditorsWithHeights([half, half, half]);
                editor_columns = [
                    <Column key='Col_1' isSize='full'> {editors[0]} </Column>,
                    <Column key='Col_2' isSize='1/2'> {editors[1]} </Column>,
                    <Column key='Col_3' isSize='1/2'> {editors[2]} </Column>,
                ];
                break;

            case EditorsLayout.ThreeMainBottom:
                editors = this.createEditorsWithHeights([half, half, half]);
                editor_columns = [
                    <Column key='Col_1' isSize='1/2'> {editors[1]} </Column>,
                    <Column key='Col_2' isSize='1/2'> {editors[2]} </Column>,
                    <Column key='Col_3' isSize='full'> {editors[0]} </Column>,
                ];
                break;

            case EditorsLayout.FourEven:
                editors = this.createEditorsWithHeights([half, half, half, half]);
                editor_columns = [
                    <Column key='Col_1' isSize='1/2'>
                        {editors[0]}
                        {editors[2]}
                    </Column>,
                    <Column key='Col_2' isSize='1/2'>
                        {editors[1]}
                        {editors[3]}
                    </Column>,
                ];
                break;

            default:
                editors = <div><p>Unknown layout.</p></div>;
        }


        return(
            <div>
                <Columns isGapless={true} className="no-margins" isMultiline={true}>
                    {editor_columns}
                </Columns>

                <Columns isGapless={true}>
                    <Column>
                        <MessageArea shader={current_shader} />
                    </Column>
                </Columns>

                <Columns>
                    <ColorPickerColumns
                        shader={current_shader}
                        onChangeLift={this.props.onChange_ColorPickerColumns}
                    />
                    <PositionSlidersColumns
                        shader={current_shader}
                        onChangeLift={this.props.onChange_PositionSlidersColumns}
                    />
                    <SliderColumns
                        shader={current_shader}
                        onChangeLift={this.props.onChange_SliderColumns}
                    />
                </Columns>
            </div>
        );
    }
}

type PM_Props = {
    shader: Shader,
    editorIdx: number,
    editorHeight: number,
    shaderEditors: ShaderEditors,
    onChangeLift: (newShader: Shader) => void,
    onBlurLift: (editorIdx: number, viewState: {}) => void,
    onFocusLift: (editorIdx: number) => void,
    monacoDidInit: bool,
    onMonacoDidInit: () => void,
};

type PM_State = {
    editor: MonacoEditor,
    monaco: any,// FIXME
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
            monaco: null,
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

        editor.onDidFocusEditorText(() => {
            this.props.onFocusLift(this.props.editorIdx);
        });

        editor.onDidBlurEditorText(() => {
            this.props.onBlurLift(
                this.props.editorIdx,
                this.state.editor.saveViewState(),
            );
        });

        let vs = this.props.shader.saved_view_state;
        if (vs !== null && typeof vs !== 'undefined') {
            editor.restoreViewState(vs);
        }

        if (this.props.shaderEditors.current_editor_idx === this.props.editorIdx) {
            editor.focus();
        }

        this.setState({
            editor: editor,
            monaco: monaco,
            modelVersions: modelVersions,
        });
    }

    onChangeLocal = (newValue: string, e: MessageEvent) => {
        let new_shader = this.props.shader;
        new_shader.content = newValue;
        this.props.onChangeLift(new_shader);
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

    editorWillMount = (monaco) => {
        // make sure to run the init calls only once
        if (!this.props.monacoDidInit && this.props.editorIdx === 0) {
            monaco.languages.register({ id: 'glsl' });
            monaco.languages.setMonarchTokensProvider('glsl', GlslTokensProvider);
            monaco.languages.registerCompletionItemProvider('glsl', GlslCompletionProvider);
            monaco.languages.registerHoverProvider('glsl', GlslHoverProvider);
            monaco.editor.defineTheme('glsl-base16-default-dark', ThemeBase16DefaultDark);
            this.props.onMonacoDidInit();
        }
    }

    componentDidUpdate(prevProps) {
        if (this.state.editor !== null && this.state.monaco !== null) {

            // if the user selected a new shader to be displayed in this editor
            if (this.props.shader.source_idx !== prevProps.shader.source_idx) {

                // and if this is the current editor, return focus to it
                if (this.props.shaderEditors.current_editor_idx === this.props.editorIdx) {
                    this.state.editor.focus();
                }

                // if the selected shader has a saved view, restore it
                let vs = this.props.shader.saved_view_state;
                if (vs !== null && typeof vs !== 'undefined') {
                    this.state.editor.restoreViewState(this.props.shader.saved_view_state);
                }

            }

            // Clear or add decorations.

            // Compare props to avoid infinite loop.

            // `this.shader.error_data.id` is the same as in `prevProps`, so we save the prev errors
            // to an attribute instead.

            let curr = this.props.shader.error_data;
            let prev = this.props.shader.prev_error_data;

            if ((curr === null || typeof curr === 'undefined')
                && (prev !== null && typeof prev !== 'undefined')) {
                // If the new prop is null but the prev. prop is not null:
                // Clear the decorations.

                let ids = this.state.editor.deltaDecorations(
                    this.props.shader.decoration_ids,
                    [],
                );

                let new_shader = this.props.shader;
                new_shader.prev_error_data = new_shader.error_data;
                new_shader.decoration_ids = ids;
                this.props.onChangeLift(new_shader);

            } else if ((prev === null || typeof prev === 'undefined')
                && (curr !== null && typeof curr !== 'undefined')) {
                // If the prev. prop is null but the new prop is not null:
                // Add all the new decorations.

                let errors: ShaderErrorMessage[] = parseShaderErrorText(curr.text);
                let ids = this.state.editor.deltaDecorations(
                    [],
                    errorsToDecorations(errors, this.state.monaco),
                );

                let new_shader = this.props.shader;
                new_shader.prev_error_data = new_shader.error_data;
                new_shader.decoration_ids = ids;
                this.props.onChangeLift(new_shader);

            } else if (curr !== null && typeof curr !== 'undefined') {
                if (prev !== null && typeof prev !== 'undefined') {

                    // If both the new- and prev. props are not null:

                    if (curr.id !== prev.id) {
                        // If the error id has changed, update the decorations from an updated error message.

                        let errors: ShaderErrorMessage[] = parseShaderErrorText(curr.text);
                        let ids = this.state.editor.deltaDecorations(
                            this.props.shader.decoration_ids,
                            errorsToDecorations(errors, this.state.monaco),
                        );

                        let new_shader = this.props.shader;
                        new_shader.prev_error_data = new_shader.error_data;
                        new_shader.decoration_ids = ids;
                        this.props.onChangeLift(new_shader);

                    } else if (this.props.shader.source_idx !== prevProps.shader.source_idx) {
                        // if the selected shader was changed,
                        // add decorations, no need to save new ids.

                        let errors: ShaderErrorMessage[] = parseShaderErrorText(curr.text);
                        this.state.editor.deltaDecorations(
                            this.props.shader.decoration_ids,
                            errorsToDecorations(errors, this.state.monaco),
                        );

                    }
                }
            }
        }
    }

    render() {
        const options = {
            language: "glsl",
            theme: "glsl-base16-default-dark",
            lineNumbers: "on",
            roundedSelection: false,
            scrollBeyondLastLine: true,
            fontFamily: "Iosevka Term Web",
            fontSize: "13px",
        };

        let is_selected = this.props.editorIdx === this.props.shaderEditors.current_editor_idx;

        return (
            <div>
              <MonacoEditor
                height={this.props.editorHeight}
                language="glsl"
                theme="glsl-base16-default-dark"
                value={this.props.shader.content}
                options={options}
                onChange={this.onChangeLocal}
                editorWillMount={this.editorWillMount}
                editorDidMount={this.editorDidMount}
              />

              <ShaderStatusBar
                  shader={this.props.shader}
                  isSelected={is_selected}
              />
          </div>
        );
    }
}

function errorsToDecorations(errors, monaco) {
    let a = errors.map((i) => {
        return {
            range: new monaco.Range(i.line_number,1, i.line_number,1),
            options: {
                isWholeLine: true,
                className: 'editor_error_line',
                linesDecorationsClassName: 'editor_error_margin',
            },
        };
    });
    return a;
}

type SSB_Props = {
    shader: Shader,
    isSelected: bool,
};

export class ShaderStatusBar extends React.Component<SSB_Props> {
    render() {

        let class_name = ["shader-name-level"];
        if (this.props.isSelected) {
            class_name.push("is-selected");
        }
        let err = this.props.shader.error_data;
        if (err !== null && typeof err !== 'undefined') {
            if (err.text.length > 0) {
                class_name.push("has-error");
            }
        }

        return (
            <Level className={class_name.join(" ")}>
                <LevelLeft>
                    <LevelItem>
                        {this.props.shader.file_path}
                    </LevelItem>
                </LevelLeft>
            </Level>
        );
    }
}

