// @flow
import React from 'react';
import { Panel, PanelHeading, Field, Label, Control, Input } from 'bloomer';
import { CurrentPage } from './Helpers';
import type { ServerMsg, DmoData, InputEvent } from './Helpers';

type DSTP_Props = {
    currentPage: number,
    onClickLift: () => void,
};

export class DmoSyncTracksPanel extends React.Component<DSTP_Props> {
    render() {
        let color = "";
        if (this.props.currentPage === CurrentPage.SyncTracks) {
            color = "primary";
        }

        return (
            <Panel onClick={this.props.onClickLift}>
                <PanelHeading hasTextColor={color}>SyncTracks</PanelHeading>
            </Panel>
        );
    }
}

type STP_Props = {
    dmoData: DmoData,
    onChangeLift: (ServerMsg) => void,
};

export class SyncTracksPage extends React.Component<STP_Props> {

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

