import React, { Component } from 'react';
//import * as ReactDOM from 'react-dom';
import { Container, Columns, Column } from 'bloomer';

import { Toolbar } from './Toolbar';
import { Sidebar } from './Sidebar';
import { TimeScrub } from './TimeScrub';

import { DmoSettingsPage } from './DmoSettings';// TODO rename to 'SettingsPage'
import { ShadersPage } from './DmoShaders';

import { CurrentPage } from './Helpers';

import './App.css';

//import logo from './logo.svg';

const PLAZMA_SERVER_PORT = 8080;

class App extends Component {
    constructor(props)
    {
        super(props);

        // TODO add a new attribute to select what is displayed in the main
        // panel.

        // NOTE No 0 index to avoid == problems.
        this.state = {
            socket: null,
            dmo_data: null,
            editor_content: null,
            current_page: CurrentPage.Shaders,
            current_shader_index: null,
            current_time: 0.0,
            sentUpdateSinceChange: false,
        };

        this.updateTimerId = null;
        this.getDmoTimeTimerId = null;

        this.sendUpdatedContent = this.sendUpdatedContent.bind(this);
        this.onEditorChange = this.onEditorChange.bind(this);

        this.onDmoShadersMenuChange = this.onDmoShadersMenuChange.bind(this);
        this.onDmoSettingsFormChange = this.onDmoSettingsFormChange.bind(this);

        this.onTimeScrubChange = this.onTimeScrubChange.bind(this);

        this.onColorPickerChange = this.onColorPickerChange.bind(this);
        this.onPositionSlidersChange = this.onPositionSlidersChange.bind(this);

        this.handleSocketOpen = this.handleSocketOpen.bind(this);
        this.handleSocketMessage = this.handleSocketMessage.bind(this);
        this.sendDmoData = this.sendDmoData.bind(this);
        this.getDmoTime = this.getDmoTime.bind(this);
    }

    componentDidMount()
    {
        const socket = new WebSocket('ws://localhost:' + PLAZMA_SERVER_PORT + '/ws/');

        socket.addEventListener('open', this.handleSocketOpen);
        socket.addEventListener('message', this.handleSocketMessage);

        this.updateTimerId = window.setInterval(this.sendDmoData, 1000);
        this.getDmoTimeTimerId = window.setInterval(this.getDmoTime, 500);

        this.setState({
            socket: socket,
        });
    }

    componentWillUnmount()
    {
        window.clearInterval(this.updateTimerId);
    }

    handleSocketOpen(event)
    {
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

    handleSocketMessage(event)
    {
        var msg = JSON.parse(event.data);
        switch (msg.data_type ) {
            case 'SetDmo':
                let d = JSON.parse(msg.data);

                let idx = this.state.current_shader_index;
                let frag_src = d.context.shader_sources[idx];
                this.setState({ dmo_data: d, editor_content: frag_src });
                this.setState({ sentUpdateSinceChange: true });
                break;

            case 'SetDmoTime':
                let time = JSON.parse(msg.data);
                this.setState({ current_time: time });
                break;

            case 'GetDmoTime':
                break;

            default:
                console.log("Error: unknown message.data_type '" + msg.data_type + "'");
        }
    }

    sendUpdatedContent(newValue)
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

    onDmoShadersMenuChange(idx)
    {
        this.setState({
            current_shader_index: idx,
            editor_content: this.state.dmo_data.context.shader_sources[idx],
        });
    }

    onDmoSettingsFormChange(msg)
    {
        if (msg.data_type === 'SetSettings') {
            this.setState({ settings: msg.data });
        }
        let server_msg = {
            data_type: 'SetSettings',
            data: JSON.stringify(msg.data),
        };
        console.log('Sending server: SetSettings');
        this.state.socket.send(JSON.stringify(server_msg));
    }

    onTimeScrubChange(msg)
    {
        if (msg.data_type === 'SetDmoTime') {
            console.log('Sending server SetDmoTime');
            this.setState({ current_time: Number(msg.data) });
            msg.data = String(msg.data);
            this.state.socket.send(JSON.stringify(msg));
        }
    }

    onEditorChange(newValue, e)
    {
        this.sendUpdatedContent(newValue);
    }

    onColorPickerChange(newValue)
    {
        this.sendUpdatedContent(newValue);
    }

    onPositionSlidersChange(newValue)
    {
        this.sendUpdatedContent(newValue);
    }

    sendDmoData()
    {
        if (this.state.sentUpdateSinceChange) {
            return;
        } else if (this.state.socket) {
            let msg = {
                data_type: 'SetDmo',
                data: JSON.stringify(this.state.dmo_data),
            };
            console.log('Sending server: SetDmo');
            this.state.socket.send(JSON.stringify(msg));
            this.setState({
                sentUpdateSinceChange: true,
            });
        }
    }

    getDmoTime()
    {
        let msg = { data_type: 'GetDmoTime', data: '' };
        this.state.socket.send(JSON.stringify(msg));
    }

    render()
    {
        let page;
        switch (this.state.current_page) {
            case CurrentPage.Settings:
                if (this.state.dmo_data === null) {
                    page = <div><p>DmoData is empty.</p></div>;
                } else {
                    page =
                        <DmoSettingsPage
                            dmoData={this.state.dmo_data}
                            onChangeLift={this.onDmoSettingsFormChange}
                        />;
                }
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
            default:
                page =
                    <div>
                        <p>no page</p>
                    </div>;
        };

        return (
            <div className="App">
                <Container>
                    <Toolbar />
                </Container>
                <Columns>
                    <Column isSize={{default: 1}}>
                        <Sidebar
                            dmoData={this.state.dmo_data}
                            currentPage={this.state.current_page}
                            currentShaderIndex={this.state.current_shader_index}

                            onClick_DmoSettingsMenu={() => this.setState({ current_page: CurrentPage.Settings })}

                            onClick_DmoShadersMenu={() => this.setState({ current_page: CurrentPage.Shaders })}
                            onChange_DmoShadersMenu={this.onDmoShadersMenuChange}
                        />
                    </Column>
                    <Column>
                        {page}
                    </Column>
                </Columns>
                <Container>
                    <TimeScrub
                        dmoData={this.state.dmo_data}
                        currentTime={this.state.current_time}
                        onChangeLift={this.onTimeScrubChange}
                    />
                </Container>
            </div>
        );
    }
}

export default App;

