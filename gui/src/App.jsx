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

import { CurrentPage, EditorsLayout } from './Helpers';
import type { ServerMsg, DmoData, Shader, ShaderEditors, ViewState } from './Helpers';

const PLAZMA_SERVER_PORT = 8080;

type AppUpdates = {
    SetDmo: bool,
    SetShader: bool,
    shaderIndexes: number[],
};

type AppState = {
    socket: ?WebSocket,
    project_root: ?string,
    dmo_data: ?DmoData,
    shaders: Shader[],
    shader_editors: ShaderEditors,
    view: ViewState,
    current_page: number,
    current_time: number,
    preview_is_open: bool,
    sentUpdateSinceChange: bool,
    updatesToSend: AppUpdates,
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
            dmo_data: null,
            shaders: [],
            shader_editors: {
                layout: EditorsLayout.ThreeMainTop,
                prev_layout: EditorsLayout.ThreeMainTop,
                full_height: 800,
                current_editor_idx: 0,
                editors: [
                    { source_idx: 2 },
                    { source_idx: 3 },
                    { source_idx: 4 },
                    { source_idx: 5 },
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
                SetDmo: false,
                SetShader: false,
                shaderIndexes: [],
            },
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
                SetDmo: false,
                SetShader: false,
                shaderIndexes: [],
            },
        });
    }

    handleSocketOpen = (event: MessageEvent) =>
    {
        console.log("Connected to server socket.");
        console.log("Send to server: FetchDmo");
        // Request DmoData from server.
        let msg: ServerMsg = {
            data_type: 'FetchDmo',
            data: '',
        };
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
            case 'SetDmo':
                let dmo_msg = JSON.parse(msg.data);
                let project_root = dmo_msg.project_root;
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
                    dmo_data: d,
                    shaders: shaders,
                    // resetUpdates
                    sentUpdateSinceChange: true,
                    updatesToSend: {
                        SetDmo: false,
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
        if (this.state.updatesToSend.SetDmo && this.state.socket) {
            let msg: ServerMsg = {
                data_type: 'SetDmo',
                data: JSON.stringify({
                    project_root: this.state.project_root,
                    dmo_data_json_str: JSON.stringify(this.state.dmo_data),
                }),
            };

            console.log('Sending server: SetDmo');
            this.sendMsgOnSocket(msg);

            let a = this.state.updatesToSend;
            a.SetDmo = false;
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

    getDmoTime = () =>
    {
        let msg: ServerMsg = { data_type: 'GetDmoTime', data: '' };
        this.sendMsgOnSocket(msg);
    }

    onKeyUp = (keyName, e, handle) => {
        let view = this.state.view;
        switch (handle.key) {

            case 'f8':
                view.time_scrub = !view.time_scrub;
                break;

            case 'f9':
                view.sidebar = !view.sidebar;
                break;

            case 'f10':
                view.toolbar = !view.toolbar;
                break;

            case 'f11':
                view.editors_only = !view.editors_only;
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
                keyName="f8,f9,f10,f11"
                onKeyDown={this.onKeyDown}
                onKeyUp={this.onKeyUp}
            >
                <div className="App">

                    <Toolbar
                        isHidden={!this.state.view.toolbar || this.state.view.editors_only}

                        view={this.state.view}

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

