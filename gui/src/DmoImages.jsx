// @flow
import React from 'react';
import { Table, Panel, PanelHeading, Field, Label, Control, Input } from 'bloomer';
import { CurrentPage } from './Helpers';
import type { ServerMsg, DmoData, InputEvent, PixelFormat } from './Helpers';

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

type II_Props = {
    idx: number,
    format: PixelFormat,
    path: string,
};

export class ImageItem extends React.Component<II_Props> {
    render()
    {
        return(
            <tr>
                <td>{this.props.idx}</td>
                <td>{this.props.format}</td>
                <td>{this.props.path}</td>
            </tr>
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
        let images = this.props.dmoData.context.index.image_paths.map((i) => {
            let path = i;
            let idx = this.props.dmoData.context.index.image_path_to_idx[path];
            let format = this.props.dmoData.context.index.image_path_to_format[path];

            return (<ImageItem path={path} idx={idx} format={format} />);
        });

        return (
            <Table>
                <thead>
                    <tr>
                        <th>idx</th>
                        <th>format</th>
                        <th>path</th>
                    </tr>
                </thead>
                <tbody>
                    {images}
                </tbody>
            </Table>
        );
    }
}
