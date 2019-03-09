import React from 'react';
import { Menu } from 'bloomer';

import { DmoShadersMenu } from './DmoShaders';
import { DmoFramebuffersMenu } from './DmoFramebuffers';
import { DmoQuadScenesMenu } from './DmoQuadScenes';
import { DmoPolygonScenesMenu } from './DmoPolygonScenes';
import { DmoImagesMenu } from './DmoImages';
import { DmoModelsMenu } from './DmoModels';
import { DmoTimelineMenu } from './DmoTimeline';
import { DmoSyncTracksMenu } from './DmoSyncTracks';
import { DmoSettingsMenu } from './DmoSettings';

// Requires props:
// - dmoData
// - currentPage
// - currentShaderIndex
// - onClick_DmoSettingsMenu
// - onClick_DmoShadersMenu
// - onChange_DmoShadersMenu
export class Sidebar extends React.Component {
    /*
    constructor(props) {
        super(props);

        this.state = {};
    }
    */

    render() {

        return (
            <Menu>
                <DmoShadersMenu
                    dmoData={this.props.dmoData}
                    currentPage={this.props.currentPage}
                    currentIndex={this.props.currentShaderIndex}
                    onClickLift={this.props.onClick_DmoShadersMenu}
                    onChangeLift={this.props.onChange_DmoShadersMenu}
                />

                <DmoFramebuffersMenu
                    //dmoData={this.props.dmoData}
                    currentPage={this.props.currentPage}
                    //currentIndex={this.props.currentFramebufferIndex}// TODO
                    onClickLift={this.props.onClick_DmoFramebuffersMenu}
                    //onChangeLift={this.props.onChange_DmoFramebuffersMenu}// TODO
                />

                <DmoQuadScenesMenu
                    currentPage={this.props.currentPage}
                    onClickLift={this.props.onClick_DmoQuadScenesMenu}
                />

                <DmoPolygonScenesMenu
                    currentPage={this.props.currentPage}
                    onClickLift={this.props.onClick_DmoPolygonScenesMenu}
                />

                <DmoImagesMenu
                    currentPage={this.props.currentPage}
                    onClickLift={this.props.onClick_DmoImagesMenu}
                />

                <DmoModelsMenu
                    currentPage={this.props.currentPage}
                    onClickLift={this.props.onClick_DmoModelsMenu}
                />

                <DmoTimelineMenu
                    currentPage={this.props.currentPage}
                    onClickLift={this.props.onClick_DmoTimelineMenu}
                />

                <DmoSyncTracksMenu
                    currentPage={this.props.currentPage}
                    onClickLift={this.props.onClick_DmoSyncTracksMenu}
                />

                <DmoSettingsMenu
                    currentPage={this.props.currentPage}
                    onClickLift={this.props.onClick_DmoSettingsMenu}
                />

            </Menu>
        );
    }
}
