// @flow
import React from 'react';
import { Column, Columns, Panel, PanelHeading, Field, Label, Control, Input, Checkbox } from 'bloomer';
import { CurrentPage } from './Helpers';
import type { ServerMsg, DmoData, InputEvent } from './Helpers';

type DSP_Props = {
    currentPage: number,
    onClickLift: () => void,
};

export class DmoSettingsPanel extends React.Component<DSP_Props> {
    render() {
        let color = "";
        if (this.props.currentPage === CurrentPage.Settings) {
            color = "primary";
        }
        return (
            <Panel onClick={this.props.onClickLift}>
                <PanelHeading hasTextColor={color}>Settings</PanelHeading>
            </Panel>
        );
    }
}

type SP_Props = {
    dmoData: DmoData,
    onChangeLift: (ServerMsg) => void,
};

export class SettingsPage extends React.Component<SP_Props> {

    onChangeLocal = (e: InputEvent) => {
        let s = this.props.dmoData.settings;
        switch(e.currentTarget.type) {
            case 'number':
                s[e.currentTarget.name] = Number(e.currentTarget.value);
                break;
            case 'checkbox':
                s[e.currentTarget.name] = e.currentTarget.checked;
                break;
            default:
                s[e.currentTarget.name] = e.currentTarget.value;
        }

        let msg: ServerMsg = {
            data_type: 'SetSettings',
            data: JSON.stringify(s),
        };
        this.props.onChangeLift(msg);
    }

    render() {
        return (
            <Columns>
                <Column>

                    <Field>
                        <Label>Mouse sensitivity</Label>
                        <Control>
                            <Input
                                name='mouse_sensitivity'
                                value={this.props.dmoData.settings.mouse_sensitivity}
                                type="number" min="0.0" step="0.1"
                                onChange={this.onChangeLocal}
                            />
                        </Control>
                    </Field>

                    <Field>
                        <Label>Movement sensitivity</Label>
                        <Control>
                            <Input
                                name='movement_sensitivity'
                                value={this.props.dmoData.settings.movement_sensitivity}
                                type="number" min="0.0" step="0.1"
                                onChange={this.onChangeLocal}
                            />
                        </Control>
                    </Field>

                </Column>
                <Column>

                    <Field>
                        <Label>Total length</Label>
                        <Control>
                            <Input
                                name='total_length'
                                value={this.props.dmoData.settings.total_length}
                                type="number" min="0.0" step="1.0"
                                onChange={this.onChangeLocal}
                            />
                        </Control>
                    </Field>

                    <Field>
                        <Control>
                            <Checkbox
                                name='start_full_screen'
                                checked={this.props.dmoData.settings.start_full_screen}
                                onChange={this.onChangeLocal}
                            >
                                Start full screen
                            </Checkbox>
                        </Control>
                    </Field>

                    <Field>
                        <Control>
                            <Checkbox
                                name='audio_play_on_start'
                                checked={this.props.dmoData.settings.audio_play_on_start}
                                onChange={this.onChangeLocal}
                            >
                                Play audio on start
                            </Checkbox>
                        </Control>
                    </Field>

                </Column>
            </Columns>
        );
    }
}
