// @flow
import React from 'react';
import { Table, Panel, PanelHeading } from 'bloomer';
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

type MI_Props = {
    model_idx: number,
    model_name: string,
    // FIXME add props
};

export class ModelItem extends React.Component<MI_Props> {
    render()
    {
        return(
            <tr>
                <td>{this.props.model_idx}</td>
                <td>{this.props.model_name}</td>
            </tr>
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
        let models = Object.keys(this.props.dmoData.context.index.model_name_to_idx).map((name) => {
            let model_idx = this.props.dmoData.context.index.model_name_to_idx[name];
            let model_name = name;
            // TODO other model properties

            return (
                <ModelItem
                    key={model_name}
                    model_idx={model_idx}
                    model_name={model_name}
                />
            );
        });

        return (
            <Table>
                <thead>
                    <tr>
                        <th>model_idx</th>
                        <th>model_name</th>
                    </tr>
                </thead>
                <tbody>
                    {models}
                </tbody>
            </Table>
        );
    }
}
