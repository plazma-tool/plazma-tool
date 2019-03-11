// @flow
import React from 'react';
import { Field, Label, Control, Input, Panel, PanelHeading } from 'bloomer';
import { CurrentPage } from './Helpers';
import type { ServerMsg, DmoData, InputEvent } from './Helpers';

type DFP_Props = {
    currentPage: number,
    onClickLift: () => void,
};

export class DmoFramebuffersPanel extends React.Component<DFP_Props> {
    render() {
        let color = "";
        if (this.props.currentPage === CurrentPage.Framebuffers) {
            color = "primary";
        }

        return (
            <Panel onClick={this.props.onClickLift}>
                <PanelHeading hasTextColor={color}>Framebuffers</PanelHeading>
            </Panel>
        );
    }
}

type FP_Props = {
    dmoData: DmoData,
    onChangeLift: (ServerMsg) => void,
};

export class FramebuffersPage extends React.Component<FP_Props> {

    onChangeLocal = (e: InputEvent) => {
        let msg: ServerMsg = {
            data_type: 'TODO: compose the message',
            data: '',
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

            </div>
        );
    }
}


