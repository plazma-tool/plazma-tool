import React from 'react';
import { Title, MenuLabel, Field, Label, Control, Input, Checkbox } from 'bloomer';
import { CurrentPage } from './Helpers';

// TODO Click on the label shows the settings form in the main panel.

// Requires props:
// - currentPage
// - onClickLift
export class DmoSettingsMenu extends React.Component {
    render() {
        let color = "";
        if (this.props.currentPage === CurrentPage.Settings) {
            color = "primary";
        }
        return (
            <Title tag='h1' hasTextColor={color} onClick={this.props.onClickLift}>
                Settings
            </Title>
        );
    }
}

// Requires props:
// - dmoData
// - onChangeLift
export class DmoSettingsForm extends React.Component {
    constructor(props) {
        super(props);
        this.onChangeLocal = this.onChangeLocal.bind(this);
    }

    onChangeLocal(e) {
        let s = this.props.dmoData.settings;
        switch(e.target.type) {
            case 'number':
                s[e.target.name] = Number(e.target.value);
                break;
            case 'checkbox':
                s[e.target.name] = e.target.checked;
                break;
            default:
                s[e.target.name] = e.target.value;
        }

        let msg = {
            data_type: 'SetSettings',
            data: s,
        };
        this.props.onChangeLift(msg);
    }

    render() {
        return (
            <div>

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

            </div>
        );
    }
}
