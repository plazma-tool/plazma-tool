// @flow
import React from 'react';
import { Column, Columns, Panel, PanelHeading, TextArea, Content } from 'bloomer';
import { CurrentPage } from './Helpers';
import type { ServerMsg, DmoData, InputEvent } from './Helpers';

type DDPanel_Props = {
    currentPage: number,
    onClickLift: () => void,
};

export class DmoDataPanel extends React.Component<DDPanel_Props> {
    render() {
        let color = "";
        if (this.props.currentPage === CurrentPage.DmoData) {
            color = "primary";
        }
        return (
            <Panel onClick={this.props.onClickLift}>
                <PanelHeading hasTextColor={color}>DmoData</PanelHeading>
            </Panel>
        );
    }
}

type DDPage_Props = {
    dmoData: DmoData,
    onChangeLift: (ServerMsg) => void,
};

export class DmoDataPage extends React.Component<DDPage_Props> {

    onChangeLocal = (e: InputEvent) => {
        console.log('TODO');
    }

    render() {
        return (
            <Columns>
                <Column>

                    <Content>

                        <p>This data represents how assets (shaders, images, models, etc.) and
                            graphics objects (framebuffers, quads, etc.) are connected in the shader
                            project.</p>

                    </Content>

                    <TextArea
                        value={JSON.stringify(this.props.dmoData)}
                        style={{height: '1000px'}}
                        readOnly
                    />

                </Column>
            </Columns>
        );
    }
}

