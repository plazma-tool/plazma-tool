// @flow
import React from 'react';
import { Panel, PanelHeading, Field, Label, Control, Input } from 'bloomer';
import { CurrentPage } from './Helpers';
import type { ServerMsg, DmoData, InputEvent } from './Helpers';

type DIP_Props = {
    currentPage: number,
    onClickLift: () => void,
};

export class DmoImagesPanel extends React.Component<DIP_Props> {
    render() {
        let color = "";
        if (this.props.currentPage === CurrentPage.Images) {
            color = "primary";
        }

        return (
            <Panel onClick={this.props.onClickLift}>
                <PanelHeading hasTextColor={color}>Images</PanelHeading>
            </Panel>
        );
    }
}

type IP_Props = {
    dmoData: DmoData,
    onChangeLift: (ServerMsg) => void,
};

export class ImagesPage extends React.Component<IP_Props> {

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
