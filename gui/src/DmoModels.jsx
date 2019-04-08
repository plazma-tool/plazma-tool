// @flow
import React from 'react';
import { Panel, PanelHeading, Field, Label, Control, Input } from 'bloomer';
import { CurrentPage } from './Helpers';
import type { ServerMsg, DmoData, InputEvent } from './Helpers';

type DMP_Props = {
    currentPage: number,
    onClickLift: () => void,
};

export class DmoModelsPanel extends React.Component<DMP_Props> {
    render() {
        let color = "";
        if (this.props.currentPage === CurrentPage.Models) {
            color = "primary";
        }

        return (
            <Panel onClick={this.props.onClickLift}>
                <PanelHeading hasTextColor={color}>Models</PanelHeading>
            </Panel>
        );
    }
}

type MP_Props = {
    dmoData: DmoData,
    onChangeLift: (ServerMsg) => void,
};

export class ModelsPage extends React.Component<MP_Props> {

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
