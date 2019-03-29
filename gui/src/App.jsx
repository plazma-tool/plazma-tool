// @flow
import React, { Component } from 'react';
//import * as ReactDOM from 'react-dom';
import { Columns, Column } from 'bloomer';

import { Toolbar } from './Toolbar';
import { Sidebar } from './Sidebar';
import { TimeScrub } from './TimeScrub';

import { SettingsPage } from './DmoSettings';
import { ShadersPage } from './DmoShaders';
import { FramebuffersPage } from './DmoFramebuffers';
import { QuadScenesPage } from './DmoQuadScenes';
import { PolygonScenesPage } from './DmoPolygonScenes';
import { ImagesPage } from './DmoImages';
import { ModelsPage } from './DmoModels';
import { TimelinePage } from './DmoTimeline';
import { SyncTracksPage } from './DmoSyncTracks';

import { LibraryPage } from './Library';
import { CurrentPage } from './Helpers';
import type { ServerMsg, DmoData } from './Helpers';

const PLAZMA_SERVER_PORT = 8080;

type AppState = {
    socket: ?WebSocket,
    dmo_data: ?DmoData,
    editor_content: string,
    current_page: number,
    current_shader_index: number,
    current_time: number,
    preview_is_open: bool,
    sentUpdateSinceChange: bool,
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
            dmo_data: null,
            editor_content: "",
            current_page: CurrentPage.Shaders,
            current_shader_index: 0,
            current_time: 0.0,
            preview_is_open: false,
            sentUpdateSinceChange: false,
        };
    }

    componentDidMount()
    {

        this.connectToServerTimerId = window.setInterval(this.connectToServer, 1000);
        this.updateTimerId = window.setInterval(this.sendDmoData, 1000);
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

                this.setState({
                    socket: socket,
                });
            }

        } else {

            // First attempt. Could be refused if server hasn't finished starting up.

            console.log("Connecting to server on port " + PLAZMA_SERVER_PORT + " ...");
            const socket = new WebSocket('ws://localhost:' + PLAZMA_SERVER_PORT + '/ws/');

            socket.addEventListener('open', this.handleSocketOpen);
            socket.addEventListener('message', this.handleSocketMessage);

            this.setState({
                socket: socket,
            });

        }
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
        this.setState({
            sentUpdateSinceChange: true,
        });
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
        switch (msg.data_type) {
            case 'SetDmo':
                let d: DmoData = JSON.parse(msg.data);

                let idx = this.state.current_shader_index;
                let frag_src = d.context.shader_sources[idx];
                this.setState({ dmo_data: d, editor_content: frag_src });
                this.setState({ sentUpdateSinceChange: true });
                break;

            case 'SetDmoTime':
                let time: number = JSON.parse(msg.data);
                this.setState({ current_time: time });
                break;

            case 'GetDmoTime':
                break;

            case 'PreviewOpened':
                this.setState({ preview_is_open: true });
                break;

            case 'PreviewClosed':
                this.setState({ preview_is_open: false });
                break;

            default:
                console.log("Error: unknown message.data_type '" + msg.data_type + "'");
        }
    }

    sendUpdatedContent = (newValue: string) =>
    {
        if (this.state.dmo_data) {
            let d = this.state.dmo_data;
            let idx = this.state.current_shader_index;
            d.context.shader_sources[idx] = newValue;

            this.setState({
                dmo_data: d,
                editor_content: newValue,
            });
        }
        this.setState({
            sentUpdateSinceChange: false,
        });
    }

    onDmoShadersMenuChange = (idx: number) =>
    {
        if (this.state.dmo_data !== null && typeof this.state.dmo_data !== 'undefined') {
            this.setState({
                current_shader_index: idx,
                editor_content: this.state.dmo_data.context.shader_sources[idx],
            });
        }
    }

    onChange_SettingsPage = (msg: ServerMsg) =>
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

    onChange_ShadersPage = (msg: ServerMsg) =>
    {
        console.log("TODO: implement onChange_ShadersPage(msg)");
    }

    onChange_FramebuffersPage = (msg: ServerMsg) =>
    {
        console.log("TODO: implement onChange_FramebuffersPage(msg)");
    }

    onChange_QuadScenesPage = (msg: ServerMsg) =>
    {
        console.log("TODO: implement onChange_QuadScenesPage(msg)");
    }

    onChange_PolygonScenesPage = (msg: ServerMsg) =>
    {
        console.log("TODO: implement onChange_PolygonScenesPage(msg)");
    }

    onChange_ImagesPage = (msg: ServerMsg) =>
    {
        console.log("TODO: implement onChange_ImagesPage(msg)");
    }

    onChange_ModelsPage = (msg: ServerMsg) =>
    {
        console.log("TODO: implement onChange_ModelsPage(msg)");
    }

    onChange_TimelinePage = (msg: ServerMsg) =>
    {
        console.log("TODO: implement onChange_TimelinePage(msg)");
    }

    onChange_SyncTracksPage = (msg: ServerMsg) =>
    {
        console.log("TODO: implement onChange_SyncTracksPage(msg)");
    }

    onChange_LibraryPage = (msg: ServerMsg) =>
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

    onEditorChange = (newValue: string, e: MessageEvent) =>
    {
        this.sendUpdatedContent(newValue);
    }

    onColorPickerChange = (newValue: string) =>
    {
        this.sendUpdatedContent(newValue);
    }

    onPositionSlidersChange = (newValue: string) =>
    {
        this.sendUpdatedContent(newValue);
    }

    sendDmoData = () =>
    {
        if (this.state.sentUpdateSinceChange) {
            return;
        } else if (this.state.socket) {
            let msg: ServerMsg = {
                data_type: 'SetDmo',
                data: JSON.stringify(this.state.dmo_data),
            };
            console.log('Sending server: SetDmo');
            this.sendMsgOnSocket(msg);
            this.setState({
                sentUpdateSinceChange: true,
            });
        }
    }

    getDmoTime = () =>
    {
        let msg: ServerMsg = { data_type: 'GetDmoTime', data: '' };
        this.sendMsgOnSocket(msg);
    }

    render()
    {
        let page;
        if (this.state.dmo_data === null || typeof this.state.dmo_data === 'undefined') {

            page = <div><p>DmoData is empty.</p></div>;

        } else {

            switch (this.state.current_page) {

                case CurrentPage.Settings:
                    page =
                        <SettingsPage
                            dmoData={this.state.dmo_data}
                            onChangeLift={this.onChange_SettingsPage}
                        />;
                    break;

                case CurrentPage.Shaders:
                    page =
                        <ShadersPage
                            editorContent={this.state.editor_content}
                            onChange_PlazmaMonaco={this.onEditorChange}
                            onChange_ColorPickerColumns={this.onColorPickerChange}
                            onChange_PositionSlidersColumns={this.onPositionSlidersChange}
                            onChange_SliderColumns={this.onColorPickerChange}
                        />
                        break;

                case CurrentPage.Framebuffers:
                    page =
                        <FramebuffersPage
                            dmoData={this.state.dmo_data}
                            onChangeLift={this.onChange_FramebuffersPage}
                        />;
                    break;

                case CurrentPage.QuadScenes:
                    page =
                        <QuadScenesPage
                            dmoData={this.state.dmo_data}
                            onChangeLift={this.onChange_QuadScenesPage}
                        />;
                    break;

                case CurrentPage.PolygonScenes:
                    page =
                        <PolygonScenesPage
                            dmoData={this.state.dmo_data}
                            onChangeLift={this.onChange_PolygonScenesPage}
                        />;
                    break;

                case CurrentPage.Images:
                    page =
                        <ImagesPage
                            dmoData={this.state.dmo_data}
                            onChangeLift={this.onChange_ImagesPage}
                        />;
                    break;

                case CurrentPage.Models:
                    page =
                        <ModelsPage
                            dmoData={this.state.dmo_data}
                            onChangeLift={this.onChange_ModelsPage}
                        />;
                    break;

                case CurrentPage.Timeline:
                    page =
                        <TimelinePage
                            dmoData={this.state.dmo_data}
                            onChangeLift={this.onChange_TimelinePage}
                        />;
                    break;

                case CurrentPage.SyncTracks:
                    page =
                        <SyncTracksPage
                            dmoData={this.state.dmo_data}
                            onChangeLift={this.onChange_SyncTracksPage}
                        />;
                    break;

                case CurrentPage.Library:
                    page =
                        <LibraryPage
                            dmoData={this.state.dmo_data}
                            onChangeLift={this.onChange_LibraryPage}
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
            <div className="App">

                <Toolbar
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
                />

                <Columns>
                    <Column isSize={{default: 2}}>
                        <Sidebar
                            dmoData={this.state.dmo_data}
                            currentPage={this.state.current_page}
                            currentShaderIndex={this.state.current_shader_index}

                            onClick_DmoSettingsMenu={() => this.setState({ current_page: CurrentPage.Settings })}
                            onClick_DmoFramebuffersMenu={() => this.setState({ current_page: CurrentPage.Framebuffers })}
                            onClick_DmoQuadScenesMenu={() => this.setState({ current_page: CurrentPage.QuadScenes })}
                            onClick_DmoPolygonScenesMenu={() => this.setState({ current_page: CurrentPage.PolygonScenes })}
                            onClick_DmoShadersMenu={() => this.setState({ current_page: CurrentPage.Shaders })}
                            onClick_DmoImagesMenu={() => this.setState({ current_page: CurrentPage.Images })}
                            onClick_DmoModelsMenu={() => this.setState({ current_page: CurrentPage.Models })}
                            onClick_DmoTimelineMenu={() => this.setState({ current_page: CurrentPage.Timeline })}
                            onClick_DmoSyncTracksMenu={() => this.setState({ current_page: CurrentPage.SyncTracks })}

                            onChange_DmoShadersMenu={this.onDmoShadersMenuChange}
                        />
                    </Column>
                    <Column>
                        {page}
                    </Column>
                </Columns>

                <TimeScrub
                    dmoData={this.state.dmo_data}
                    currentTime={this.state.current_time}
                    onChangeLift={this.onTimeScrubChange}
                />

            </div>
        );
    }
}

export default App;

