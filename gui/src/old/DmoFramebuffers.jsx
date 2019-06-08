// @flow
import React from 'react';
import { Columns, Column, Image, Field, FieldBody, Icon, Box, Label, Control, Input, Select, Panel, PanelHeading } from 'bloomer';
import { CurrentPage } from './Helpers';
import type { ServerMsg, DmoData, FrameBuffer, InputEvent } from './Helpers';

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

type FBI_Props = {
    data: FrameBuffer,
};

class FrameBufferItem extends React.Component<FBI_Props> {
    render() {
        let image = <Image isSize="128x128" src="" />;

        return(
            <Box>
                <Columns>
                    <Column>

                        <Field isHorizontal>
                            <FieldBody>
                                <Field isGrouped>
                                    <Control isExpanded hasIcons='left'>
                                        <Input placeholder='buffer name' value={this.props.data.name} />
                                        <Icon isSize='small' isAlign='left'><span className="fa fa-user" aria-hidden="true"/></Icon>
                                    </Control>
                                </Field>

                                <Field isGrouped>
                                    <Label>Kind:</Label>
                                    <Control>
                                        <Select value={this.props.data.kind}>
                                            {/* <option>NOOP</option> */}
                                            <option>Empty_Texture</option>
                                            <option>Image_Texture</option>
                                        </Select>
                                    </Control>
                                </Field>

                                <Field isGrouped>
                                    <Label>Format:</Label>
                                    <Control>
                                        <Select value={this.props.data.format}>
                                            {/* <option>NOOP</option> */}
                                            <option>RED_u8</option>
                                            <option>RGB_u8</option>
                                            <option>RGBA_u8</option>
                                        </Select>
                                    </Control>
                                </Field>

                            </FieldBody>
                        </Field>

                        <Field>
                            <Control isExpanded hasIcons='left'>
                                <div className="file has-name is-fullwidth">
                                    <label className="file-label">
                                        <Input className="file-input" type="file" name="resume" />
                                        <span className="file-cta">
                                            <span className="file-icon">
                                                <i className="fas fa-upload"></i>
                                            </span>
                                            <span className="file-label">
                                                Choose a file…
                                            </span>
                                        </span>
                                        <span className="file-name">
                                            {this.props.data.image_path}
                                        </span>
                                    </label>
                                </div>
                            </Control>
                        </Field>

                    </Column>

                    <Column isSize="1/3">
                        {image}
                    </Column>

                </Columns>

            </Box>
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
        let frame_buffers = this.props.dmoData.context.frame_buffers.map((i) => {
            return <FrameBufferItem data={i} key={i.name} />;
        });

        return (
            <div>
                {frame_buffers}
            </div>
        );
    }
}


