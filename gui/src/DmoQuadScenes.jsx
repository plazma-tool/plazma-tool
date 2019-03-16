// @flow
import React from 'react';
import { Table, FieldBody, Icon, Select, Box, Column, Columns, Panel, PanelHeading, Field, Label, Control, Input } from 'bloomer';
import { CurrentPage } from './Helpers';
import type { ServerMsg, DmoData, InputEvent, QuadScene } from './Helpers';

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

type QSI_Props = {
    data: QuadScene,
};

class QuadSceneItem extends React.Component<QSI_Props> {
    render()
    {
        //console.log('layout_to_vars');
        //console.log(this.props.data.layout_to_vars);

        let layout_to_vars = this.props.data.layout_to_vars.map((i) => {
            let fields = "";

            if (i === "NOOP") {
                // noop.
            } else if (typeof i === 'object') {

                if (i && i.Float && typeof i.Float === 'object') {

                    fields =
                        <tr>
                            <td>

                                <Field>
                                    <Control>
                                        <Input
                                            value={i.Float[0]}
                                            type="number" min="0" step="1"
                                        />
                                    </Control>
                                </Field>

                            </td>
                            <td>

                                <Field>
                                    <Control>
                                        <Select value={i.Float[1]}>
                                            <option>Time</option>
                                            <option>Window_Width</option>
                                            <option>Window_Height</option>
                                            <option>Screen_Width</option>
                                            <option>Screen_Height</option>
                                            <option>Camera_Pos_X</option>
                                            <option>Camera_Pos_Y</option>
                                            <option>Camera_Pos_Z</option>
                                        </Select>
                                    </Control>
                                </Field>

                            </td>

                            <td>--</td>
                            <td>--</td>
                            <td>--</td>
                        </tr>;

                } else if (i && i.Vec2 && typeof i.Vec2 === 'object') {

                    fields =
                        <tr>
                            <td>

                                <Field>
                                    <Control>
                                        <Input
                                            value={i.Vec2[0]}
                                            type="number" min="0" step="1"
                                        />
                                    </Control>
                                </Field>

                            </td>
                            <td>

                                <Field>
                                    <Control>
                                        <Select value={i.Vec2[1]}>
                                            <option>Time</option>
                                            <option>Window_Width</option>
                                            <option>Window_Height</option>
                                            <option>Screen_Width</option>
                                            <option>Screen_Height</option>
                                            <option>Camera_Pos_X</option>
                                            <option>Camera_Pos_Y</option>
                                            <option>Camera_Pos_Z</option>
                                        </Select>
                                    </Control>
                                </Field>

                            </td>
                            <td>

                                <Field>
                                    <Control>
                                        <Select value={i.Vec2[2]}>
                                            <option>Time</option>
                                            <option>Window_Width</option>
                                            <option>Window_Height</option>
                                            <option>Screen_Width</option>
                                            <option>Screen_Height</option>
                                            <option>Camera_Pos_X</option>
                                            <option>Camera_Pos_Y</option>
                                            <option>Camera_Pos_Z</option>
                                        </Select>
                                    </Control>
                                </Field>

                            </td>

                            <td>--</td>
                            <td>--</td>
                        </tr>;

                } else if (i && i.Vec3 && typeof i.Vec3 === 'object') {

                } else if (i && i.Vec4 && typeof i.Vec4 === 'object') {

                }

            }

            return fields;
        });

        let binding_to_buffers = this.props.data.binding_to_buffers.map((i) => {
            let fields = "";

            if (i === "NOOP") {
                // noop.
            } else if (typeof i === 'object') {

                if (Object.getOwnPropertyNames(i).includes("Sampler2D")) {
                    fields =
                        <tr>
                            <td>

                                <Field>
                                    <Control>
                                        <Input
                                            value={i["Sampler2D"][0]}
                                            type="number" min="0" step="1"
                                        />
                                    </Control>
                                </Field>

                            </td>
                            <td>

                                <Field>
                                    <Control>
                                        <Input
                                            value={i["Sampler2D"][1]}
                                            placeholder="buffer name"
                                            type="text"
                                        />
                                    </Control>
                                </Field>

                            </td>
                        </tr>;
                }
            }

            return fields;
        });

        return(
            <Box>

                <Field isHorizontal>
                    <FieldBody>

                        <Columns>
                            <Column>
                                <Field isGrouped>
                                    <Control isExpanded hasIcons='left'>
                                        <Input placeholder='scene name' value={this.props.data.name} />
                                        <Icon isSize='small' isAlign='left'><span className="fa fa-user" aria-hidden="true"/></Icon>
                                    </Control>
                                </Field>
                            </Column>

                            <Column>
                                <Field isGrouped>
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
                                                    {this.props.data.vert_src_path}
                                                </span>
                                            </label>
                                        </div>
                                    </Control>
                                </Field>

                                <Field isGrouped>
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
                                                    {this.props.data.frag_src_path}
                                                </span>
                                            </label>
                                        </div>
                                    </Control>
                                </Field>

                            </Column>
                        </Columns>

                    </FieldBody>
                </Field>

                <Columns>
                    <Column>
                        <div>Layout to vars:</div>
                        <Table>
                            <thead>
                                <tr>
                                    <th>Layout idx</th>
                                    <th>vec[0]</th>
                                    <th>vec[1]</th>
                                    <th>vec[2]</th>
                                    <th>vec[3]</th>
                                </tr>
                            </thead>
                            <tbody>
                                {layout_to_vars}
                            </tbody>
                        </Table>
                    </Column>

                    <Column>
                        <div>Binding to buffers:</div>
                        <Table>
                            <thead>
                                <tr>
                                    <th>Binding idx</th>
                                    <th>buffer name</th>
                                </tr>
                            </thead>
                            <tbody>
                                {binding_to_buffers}
                            </tbody>
                        </Table>
                    </Column>
                </Columns>

            </Box>
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
        let quad_scenes = this.props.dmoData.context.quad_scenes.map((i) => {
            return <QuadSceneItem data={i} key={i.name} />;
        });
        return (
            <div>
                {quad_scenes}
            </div>
        );
    }
}
