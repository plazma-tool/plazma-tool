// @flow
import React, { Component } from 'react';
//import * as ReactDOM from 'react-dom';
import { Columns, Column } from 'bloomer';
import Hotkeys from 'react-hot-keys';

import { Toolbar } from './Toolbar';
import { Sidebar } from './Sidebar';
import { TimeScrub } from './TimeScrub';

import { PropertiesPage } from './DmoProperties';
import { ShadersPage } from './DmoShaders';
import { DmoDataPage } from './DmoData';
import { LibraryPage } from './Library';

import { CurrentPage, EditorsLayout, NewProjectTemplateString } from './Helpers';
import type { ServerMsg, DmoData, Shader, ShaderEditors, ViewState } from './Helpers';

const PLAZMA_SERVER_PORT = 8080;

type AppUpdates = {
    SetDmoInline: bool,
    SetShader: bool,
    shaderIndexes: number[],
};

type AppState = {
    socket: ?WebSocket,
    project_root: ?string,
    demo_yml_path: ?string,
    dmo_data: ?DmoData,
    embedded: ?bool,
    shaders: Shader[],
    shader_editors: ShaderEditors,
    view: ViewState,
    current_page: number,
    current_time: number,
    preview_is_open: bool,
    sentUpdateSinceChange: bool,
    updatesToSend: AppUpdates,
    events: {},
    new_project_modal_is_active: bool,
    import_from_shadertoy_modal_is_active: bool,
    monacoDidInit: bool
};

class App extends Component<{}, AppState> {
    updateTimerId: number;
    getDmoTimeTimerId: number;
    connectToServerTimerId: number;

    constructor(props: {})
    {
        super(props);

        this.state = {
            socket: null,
            project_root: null,
            demo_yml_path: null,
            dmo_data: null,
            embedded: null,
            shaders: [],
            shader_editors: {
                layout: EditorsLayout.OneMax,
                full_height: 800,
                current_editor_idx: 0,
                editors: [
                    { source_idx: 2 },
                    { source_idx: 2 },
                    { source_idx: 2 },
                    { source_idx: 2 },
                ],
            },
            view: {
                time_scrub: true,
                sidebar: true,
                toolbar: true,
                editors_only: false,
            },
            current_page: CurrentPage.Shaders,
            current_time: 0.0,
            preview_is_open: false,
            sentUpdateSinceChange: true,
            updatesToSend: {
                SetDmoInline: false,
                SetShader: false,
                shaderIndexes: [],
            },
            events: {
                layout_changed: new Event('layout_changed'),
            },
            new_project_modal_is_active: false,
            import_from_shadertoy_modal_is_active: false,
            monacoDidInit: false,
        };
    }

    componentDidMount()
    {
        this.connectToServerTimerId = window.setInterval(this.connectToServer, 1000);
        this.updateTimerId = window.setInterval(this.sendUpdates, 1000);
        this.getDmoTimeTimerId = window.setInterval(this.getDmoTime, 500);
    }

    componentWillUnmount()
    {
        window.clearInterval(this.updateTimerId);
        window.clearInterval(this.getDmoTimeTimerId);
        window.clearInterval(this.connectToServerTimerId);
    }

    connectToServer = () =>
    {
        if (this.state.socket !== null && typeof this.state.socket !== 'undefined') {

            if (this.state.socket.readyState === WebSocket.OPEN
                || this.state.socket.readyState === WebSocket.CONNECTING) {

                // Good to go.
                window.clearInterval(this.connectToServerTimerId);

            } else if (this.state.socket.readyState === WebSocket.CLOSED) {

                // Connection was attempted before but probably refused.
                console.log("Connecting to server on port " + PLAZMA_SERVER_PORT + " ...");
                const socket = new WebSocket('ws://localhost:' + PLAZMA_SERVER_PORT + '/ws/');

                socket.addEventListener('open', this.handleSocketOpen);
                socket.addEventListener('message', this.handleSocketMessage);

                this.setState({ socket: socket });
            }

        } else {

            // First attempt. Could be refused if server hasn't finished starting up.

            console.log("Connecting to server on port " + PLAZMA_SERVER_PORT + " ...");
            const socket = new WebSocket('ws://localhost:' + PLAZMA_SERVER_PORT + '/ws/');

            socket.addEventListener('open', this.handleSocketOpen);
            socket.addEventListener('message', this.handleSocketMessage);

            this.setState({ socket: socket });
        }
    }

    resetUpdates = () => {
        this.setState({
            sentUpdateSinceChange: true,
            updatesToSend: {
                SetDmoInline: false,
                SetShader: false,
                shaderIndexes: [],
            },
        });
    }

    handleSocketOpen = (event: MessageEvent) =>
    {
        console.log("Connected to server socket.");
        console.log("Send to server: FetchDmoInline");
        // Request DmoData from server.
        let msg: ServerMsg = { data_type: 'FetchDmoInline', data: '' };
        this.sendMsgOnSocket(msg);
        this.resetUpdates();
    }

    sendMsgOnSocket = (msg: ServerMsg) =>
    {
        if (this.state.socket !== null
            && typeof this.state.socket !== 'undefined'
            && this.state.socket.readyState === WebSocket.OPEN) {
            this.state.socket.send(JSON.stringify(msg));
        }
    }

    handleSocketMessage = (event: MessageEvent) =>
    {
        let msg: ServerMsg = { data_type: 'NoOp', data: '' };
        if (typeof event.data === 'string') {
            msg = JSON.parse(event.data);
        }

        let shaders = [];

        switch (msg.data_type) {
            case 'NoOp':
                break;

            case 'SetDmoFile':
                {
                    // The browser can't handle SetDmoFile, so request it inline
                    let m: ServerMsg = { data_type: 'FetchDmoInline', data: '' };
                    this.sendMsgOnSocket(m);
                    this.resetUpdates();
                    // The browser can't delete the file path in SetDmoFile message, so tell the server to do that
                    m = { data_type: 'DeleteMessageFile', data: msg.data };
                    this.sendMsgOnSocket(m);
                }
                break;

            case 'FetchDmoFile':
                break;

            case 'SetDmoInline':
                console.log('Received SetDmoInline.');
                let dmo_msg = JSON.parse(msg.data);
                let project_root = dmo_msg.project_root;
                let demo_yml_path = dmo_msg.demo_yml_path;
                let embedded = dmo_msg.embedded;
                let d: DmoData = JSON.parse(dmo_msg.dmo_data_json_str);

                shaders = d.context.shader_sources.map((i, idx) => {
                    return {
                        content: i,
                        file_path: d.context.index.shader_paths[idx],
                        source_idx: idx,
                        saved_view_state: null,
                        error_data: null,
                        prev_error_data: null,
                        decoration_ids: [],
                    };
                });

                this.setState({
                    project_root: project_root,
                    demo_yml_path: demo_yml_path,
                    dmo_data: d,
                    embedded: embedded,
                    shaders: shaders,
                    // resetUpdates
                    sentUpdateSinceChange: true,
                    updatesToSend: {
                        SetDmoInline: false,
                        SetShader: false,
                        shaderIndexes: [],
                    },
                });
                break;

            case 'SetDmoTime':
                let time: number = JSON.parse(msg.data);
                this.setState({ current_time: time });
                break;

            case 'GetDmoTime':
                break;

            case 'PreviewOpened':
                // clear possible old errors from shaders
                shaders = this.state.shaders.map((i) => { i.error_data = null; return i; });

                this.setState({
                    preview_is_open: true,
                    shaders: shaders,
                });
                break;

            case 'PreviewClosed':
                // clear errors from shaders since we can no longer check by compiling
                shaders = this.state.shaders.map((i) => { i.error_data = null; return i; });

                this.setState({
                    preview_is_open: false,
                    shaders: shaders,
                });
                break;

            case 'ShaderCompilationSuccess':
                if (msg.data.length > 0) {
                    // If there is data, it is the shader idx which we tried to updated. Clear the errors on it.
                    let m = JSON.parse(msg.data);
                    let s = this.state.shaders;
                    s[m.idx].error_data = null;
                    shaders = s;
                } else {
                    // Otherwise clear all errors.
                    shaders = this.state.shaders.map((i) => { i.error_data = null; return i; });
                }

                this.setState({ shaders: shaders });
                break;

            case 'ShaderCompilationFailed':
                let e = JSON.parse(msg.data);

                let src_idx = e.idx;
                let error_msg = e.error_message;

                shaders = this.state.shaders;
                let prev = shaders[src_idx].prev_error_data;

                // Should the new error data get an updated id?

                let id = 0;
                if (prev !== null && typeof prev !== 'undefined') {
                    id = (error_msg === prev.text) ? prev.id : prev.id + 1;
                }
                shaders[src_idx].error_data = {
                    id: id,
                    text: error_msg,
                };
                this.setState({ shaders: shaders });

                break;

            default:
                console.log("Error: unknown message.data_type '" + msg.data_type + "'");
        }
    }

    currentShaderIdx = () =>
    {
        let e_idx = this.state.shader_editors.current_editor_idx;
        return this.state.shader_editors.editors[e_idx].source_idx;
    }

    sendUpdatedContent = (newShader: Shader) =>
    {
        if (this.state.dmo_data) {
            let d = this.state.dmo_data;
            let src_idx = newShader.source_idx;
            d.context.shader_sources[src_idx] = newShader.content;

            let a = this.state.updatesToSend;
            a.SetShader = true;
            a.shaderIndexes.push(src_idx);

            let s = this.state.shaders;
            s[newShader.source_idx] = newShader;

            this.setState({
                dmo_data: d,
                shaders: s,
                sentUpdateSinceChange: false,
                updatesToSend: a,
            });
        }
    }

    onDmoShadersMenuChange = (idx: number) =>
    {
        if (this.state.dmo_data !== null && typeof this.state.dmo_data !== 'undefined') {
            let e_idx = this.state.shader_editors.current_editor_idx;
            let e = this.state.shader_editors;
            e.editors[e_idx].source_idx = idx;

            this.setState({ shader_editors: e });
        }
    }

    onChange_Settings = (msg: ServerMsg) =>
    {
        if (msg.data_type === 'SetSettings') {
            if (this.state.dmo_data !== null && typeof this.state.dmo_data !== 'undefined') {
                let d = this.state.dmo_data;
                d.settings = JSON.parse(msg.data);
                this.setState({ dmo_data: d });
            }
        }
        console.log('Sending server: SetSettings');
        this.sendMsgOnSocket(msg);
    }

    onChange_Metadata = (msg: ServerMsg) =>
    {
        if (msg.data_type === 'SetMetadata') {
            if (this.state.dmo_data !== null && typeof this.state.dmo_data !== 'undefined') {
                let d = this.state.dmo_data;
                d.metadata = JSON.parse(msg.data);
                this.setState({ dmo_data: d });
            }
        }
        // As it is now, Metadata doesn't have to be sent to the server since it doesn't use it. It
        // will be saved to the YAML when the project DmoData is saved.
    }

    onChange_ShadersPage = (msg: ServerMsg) =>
    {
        console.log("TODO: implement onChange_ShadersPage(msg)");
    }

    onChange_LibraryPage = (msg: ServerMsg) =>
    {
        console.log("TODO: implement onChange_LibraryPage(msg)");
    }

    onChange_DmoDataPage = (msg: ServerMsg) =>
    {
        console.log("TODO: implement onChange_LibraryPage(msg)");
    }

    onClick_NewProjectSet = (is_active: bool) => {
        this.setState({ new_project_modal_is_active: is_active });
    }

    onClick_NewProjectButton = (template: number) => {
        let msg: ServerMsg = {
            data_type: 'NewProject',
            data: JSON.stringify({ template: NewProjectTemplateString[template] }),
        };
        this.sendMsgOnSocket(msg);
    }

    onClick_OpenProject = () => {
        let msg: ServerMsg = { data_type: 'OpenProjectFileDialog', data: '' };
        this.sendMsgOnSocket(msg);
    }

    onClick_SaveProject = () => {
        let msg: ServerMsg = { data_type: 'SaveProject', data: '' };
        this.sendMsgOnSocket(msg);
    }

    onClick_ImportProjectSet = (is_active: bool) => {
        this.setState({ import_from_shadertoy_modal_is_active: is_active });
    }

    onClick_ReloadProject = () => {
        let msg: ServerMsg = { data_type: 'ReloadProject', data: '' };
        this.sendMsgOnSocket(msg);
    }

    onTimeScrubChange = (msg: ServerMsg) =>
    {
        if (msg.data_type === 'SetDmoTime') {
            console.log('Sending server SetDmoTime');
            this.setState({ current_time: Number(msg.data) });
            msg.data = String(msg.data);
            this.sendMsgOnSocket(msg);
        }
    }

    onEditorChange = (newShader: Shader) => {
        this.sendUpdatedContent(newShader);
    }

    onEditorFocus = (editorIdx: number) => {
        let s = this.state.shader_editors;
        s.current_editor_idx = editorIdx;
        this.setState({ shader_editors: s });
    }

    onEditorBlur = (editorIdx: number, viewState: ?{}) => {
        let src_idx = this.state.shader_editors.editors[editorIdx].source_idx;
        let s = this.state.shaders;
        s[src_idx].saved_view_state = viewState;
        this.setState({ shaders: s });
    }

    onEditorKey = (key: string) => {
        this.onKeyUp(key, {}, { key: key });
    }

    onColorPickerChange = (newShader: Shader) => {
        this.sendUpdatedContent(newShader);
    }

    onPositionSlidersChange = (newShader: Shader) => {
        this.sendUpdatedContent(newShader);
    }

    sendUpdates = () => {
        if (this.state.sentUpdateSinceChange) {
            return;
        } else if (this.state.socket) {
            this.sendDmoData();
            this.sendSetShader();
            this.resetUpdates();
        }
    }

    sendDmoData = () => {
        if (this.state.updatesToSend.SetDmoInline && this.state.socket) {
            let data: string = JSON.stringify({
                project_root: this.state.project_root,
                demo_yml_path: this.state.demo_yml_path,
                dmo_data_json_str: JSON.stringify(this.state.dmo_data),
                embedded: this.state.embedded,
            });

            // FIXME this is 50k data, not sure where the bug occurs but somewhere around 100k
            if (data.length > 50*1024) {
                console.log("FIXME not sending large SetDmoInline");
                return;
            }

            let msg: ServerMsg = {
                data_type: 'SetDmoInline',
                data: data,
            };

            console.log('Sending server: SetDmoInline');
            this.sendMsgOnSocket(msg);

            let a = this.state.updatesToSend;
            a.SetDmoInline = false;
            this.setState({
                updatesToSend: a,
            });
        }
    }

    sendSetShader = () =>
    {
        if (this.state.updatesToSend.SetShader && this.state.socket) {
            // filter the indexes.
            var filtered = (function(items){
                var m = {}, new_items = [];
                for (var i=0; i<items.length; i++) {
                    var v = items[i];
                    if (!m[v]) {
                        new_items.push(v);
                        m[v] = true;
                    }
                }
                return new_items;
            })(this.state.updatesToSend.shaderIndexes);

            filtered.forEach((idx) => {
                if (this.state.dmo_data === null || typeof this.state.dmo_data === 'undefined') {
                    console.log("ERROR: trying to use dmo_data while null");
                    return;
                } else {
                    let msg: ServerMsg = {
                        data_type: 'SetShader',
                        data: JSON.stringify({
                            idx: idx,
                            content: this.state.dmo_data.context.shader_sources[idx],
                        }),
                    };

                    console.log('Sending server: SetShader idx ' + idx);
                    this.sendMsgOnSocket(msg);
                }
            });

            let a = this.state.updatesToSend;
            a.SetShader = false;
            a.shaderIndexes = [];
            this.setState({
                updatesToSend: a,
            });
        }
    }

    getDmoTime = () => {
        let msg: ServerMsg = { data_type: 'GetDmoTime', data: '' };
        this.sendMsgOnSocket(msg);
    }

    onKeyUp = (keyName, e, handle) => {
        let view = this.state.view;
        switch (handle.key) {

            case 'ctrl+n':
                this.onClick_NewProjectSet(true);
                break;

            case 'ctrl+o':
                this.onClick_OpenProject();
                break;

            case 'ctrl+s':
                this.onClick_SaveProject();
                break;

            case 'ctrl+r':
                this.onClick_ReloadProject();
                break;

            case 'f8':
                view.time_scrub = !view.time_scrub;
                window.setTimeout(() => { window.dispatchEvent(this.state.events.layout_changed); }, 100);
                break;

            case 'f9':
                view.sidebar = !view.sidebar;
                window.setTimeout(() => { window.dispatchEvent(this.state.events.layout_changed); }, 100);
                break;

            case 'f10':
                view.toolbar = !view.toolbar;
                window.setTimeout(() => { window.dispatchEvent(this.state.events.layout_changed); }, 100);
                break;

            case 'f11':
                view.editors_only = !view.editors_only;
                window.setTimeout(() => { window.dispatchEvent(this.state.events.layout_changed); }, 100);
                break;

            case 'esc':
                this.setState({
                    new_project_modal_is_active: false,
                    import_from_shadertoy_modal_is_active: false,
                });
                break;

            default:
                break;
        }

        this.setState({ view: view });
    }

    onKeyDown = (keyName, e, handle) => {
        //console.log("onKeyDown", keyName, e, handle)
    }

    render()
    {
        let page;
        if (this.state.dmo_data === null || typeof this.state.dmo_data === 'undefined') {

            page = <div><p>DmoData is empty.</p></div>;

        } else {

            switch (this.state.current_page) {

                case CurrentPage.Library:
                    page =
                        <LibraryPage
                            dmoData={this.state.dmo_data}
                            onChangeLift={this.onChange_LibraryPage}
                        />;
                    break;

                case CurrentPage.Shaders:
                    page =
                        <ShadersPage
                            shaders={this.state.shaders}
                            shaderEditors={this.state.shader_editors}
                            onChange_PlazmaMonaco={this.onEditorChange}
                            onFocus_PlazmaMonaco={this.onEditorFocus}
                            onBlur_PlazmaMonaco={this.onEditorBlur}
                            onKey_PlazmaMonaco={this.onEditorKey}
                            onChange_ColorPickerColumns={this.onColorPickerChange}
                            onChange_PositionSlidersColumns={this.onPositionSlidersChange}
                            onChange_SliderColumns={this.onColorPickerChange}
                            monacoDidInit={this.state.monacoDidInit}
                            onMonacoDidInit={() => this.setState({ monacoDidInit: true })}
                        />
                        break;

                case CurrentPage.Properties:
                    page =
                        <PropertiesPage
                            dmoData={this.state.dmo_data}
                            onChange_Metadata={this.onChange_Metadata}
                            onChange_Settings={this.onChange_Settings}
                        />;
                    break;

                case CurrentPage.DmoData:
                    page =
                        <DmoDataPage
                            dmoData={this.state.dmo_data}
                            onChangeLift={this.onChange_DmoDataPage}
                        />;
                    break;

                default:
                    page =
                        <div>
                            <p>no page</p>
                        </div>;
            };

        }

        return (
            <Hotkeys
                keyName="ctrl+n,ctrl+o,ctrl+s,ctrl+r,f8,f9,f10,f11,esc"
                onKeyDown={this.onKeyDown}
                onKeyUp={this.onKeyUp}
            >
                <div className="App">

                    <Toolbar
                        isHidden={!this.state.view.toolbar || this.state.view.editors_only}

                        view={this.state.view}

                        onClick_NewProjectSet={this.onClick_NewProjectSet}
                        onClick_NewProjectButton={this.onClick_NewProjectButton}
                        onClick_OpenProject={this.onClick_OpenProject}
                        onClick_ImportProjectSet={this.onClick_ImportProjectSet}
                        onClick_ReloadProject={this.onClick_ReloadProject}
                        onClick_SaveProject={this.onClick_SaveProject}

                        new_project_modal_is_active={this.state.new_project_modal_is_active}
                        import_from_shadertoy_modal_is_active={this.state.import_from_shadertoy_modal_is_active}

                        onClick_Library={() => this.setState({ current_page: CurrentPage.Library })}

                        onClick_Preview={() => {
                            if (this.state.preview_is_open) {

                                console.log("Send to server: StopPreview");
                                let m: ServerMsg = {
                                    data_type: 'StopPreview',
                                    data: '',
                                };
                                this.sendMsgOnSocket(m);

                            } else {

                                console.log("Send to server: StartPreview");
                                let m: ServerMsg = {
                                    data_type: 'StartPreview',
                                    data: '',
                                };
                                this.sendMsgOnSocket(m);

                            }
                        } }

                        onClick_Exit={() => {
                            console.log("Send to server: ExitApp");
                            let m: ServerMsg = {
                                data_type: 'ExitApp',
                                data: '',
                            };
                            this.sendMsgOnSocket(m);
                        }}

                        previewIsOpen={this.state.preview_is_open}

                        currentLayout={this.state.shader_editors.layout}

                        onClick_Layout={(layout_index: number) => {
                            let e = this.state.shader_editors;
                            e.layout = layout_index;
                            this.setState({ shader_editors: e });
                        }}

                        onClick_View={(view: ViewState) => this.setState({ view: view })}
                    />

                <Columns isGapless={true}>
                    <Column
                        isSize={{default: 2}}
                        isHidden={!this.state.view.sidebar || this.state.view.editors_only}
                    >
                        <Sidebar
                            dmoData={this.state.dmo_data}
                            shaders={this.state.shaders}
                            currentPage={this.state.current_page}
                            currentShaderIndex={this.currentShaderIdx()}

                            onClick_DmoShadersMenu={() => this.setState({ current_page: CurrentPage.Shaders })}
                            onClick_DmoDataMenu={() => this.setState({ current_page: CurrentPage.DmoData })}
                            onClick_DmoPropertiesMenu={() => this.setState({ current_page: CurrentPage.Properties })}

                            onChange_DmoShadersMenu={this.onDmoShadersMenuChange}
                        />
                    </Column>
                    <Column className="editors-column">
                        {page}
                    </Column>
                </Columns>

                <TimeScrub
                    isHidden={!this.state.view.time_scrub || this.state.view.editors_only}
                    dmoData={this.state.dmo_data}
                    currentTime={this.state.current_time}
                    onChangeLift={this.onTimeScrubChange}
                />
            </div>
        </Hotkeys>
        );
    }
}

export default App;

