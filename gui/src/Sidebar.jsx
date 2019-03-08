import React from 'react';
import { Menu } from 'bloomer';

import { DmoSettingsMenu } from './DmoSettings';
//import { DmoFramebuffersMenu } from './DmoFramebuffers';
//import { DmoQuadScenesMenu } from './DmoQuadScenes';
//import { DmoPolygonScenesMenu } from './DmoPolygonScenes';
import { DmoShadersMenu } from './DmoShaders';
//import { DmoImagesMenu } from './DmoImages';
//import { DmoModelsMenu } from './DmoModels';
//import { DmoTimelineMenu } from './DmoTimeline';
//import { DmoSyncTracksMenu } from './DmoSyncTracks';

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
                <DmoSettingsMenu
                    currentPage={this.props.currentPage}
                    onClickLift={this.props.onClick_DmoSettingsMenu}
                />

                <DmoShadersMenu
                    dmoData={this.props.dmoData}
                    currentIndex={this.props.currentShaderIndex}
                    currentPage={this.props.currentPage}
                    onClickLift={this.props.onClick_DmoShadersMenu}
                    onChangeLift={this.props.onChange_DmoShadersMenu}
                />
            </Menu>
        );
    }
}
