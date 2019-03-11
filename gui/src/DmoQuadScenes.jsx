// @flow
import React from 'react';
import { Panel, PanelHeading, Field, Label, Control, Input } from 'bloomer';
import { CurrentPage } from './Helpers';
import type { ServerMsg, DmoData, InputEvent } from './Helpers';

type DQSP_Props = {
    currentPage: number,
    onClickLift: () => void,
};

export class DmoQuadScenesPanel extends React.Component<DQSP_Props> {
    render() {
        let color = "";
        if (this.props.currentPage === CurrentPage.QuadScenes) {
            color = "primary";
        }

        return (
            <Panel onClick={this.props.onClickLift}>
                <PanelHeading hasTextColor={color}>QuadScenes</PanelHeading>
            </Panel>
        );
    }
}

type QSP_Props = {
    dmoData: DmoData,
    onChangeLift: (ServerMsg) => void,
};

export class QuadScenesPage extends React.Component<QSP_Props> {

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
