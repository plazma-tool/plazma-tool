import React from 'react';
import { Menu, Button, Icon } from 'bloomer';

import { DmoShadersPanel } from './DmoShaders';
import { DmoFramebuffersPanel } from './DmoFramebuffers';
import { DmoQuadScenesPanel } from './DmoQuadScenes';
import { DmoPolygonScenesPanel } from './DmoPolygonScenes';
import { DmoImagesPanel } from './DmoImages';
import { DmoModelsPanel } from './DmoModels';
import { DmoTimelinePanel } from './DmoTimeline';
import { DmoSyncTracksPanel } from './DmoSyncTracks';
import { DmoSettingsPanel } from './DmoSettings';

// Requires props:
// - isActive
// - text
// - onClickLift
export class SidebarButton extends React.Component {
    render() {
        let color = "";
        if (this.props.isActive) {
            color = "primary";
        }

        return (
            <div onClick={this.props.onClickLift}>
                <Button hasTextColor={color} isColor="dark" className="is-fullwidth" >
                    <Icon className="fa fa-bolt" />
                    <span>{this.props.text}</span>
                </Button>
            </div>
        );
    }
}

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
            <div id="sidebar">
                <DmoShadersPanel
                    dmoData={this.props.dmoData}
                    currentPage={this.props.currentPage}
                    currentIndex={this.props.currentShaderIndex}
                    onClickLift={this.props.onClick_DmoShadersMenu}
                    onChangeLift={this.props.onChange_DmoShadersMenu}
                />

                <DmoFramebuffersPanel
                    //dmoData={this.props.dmoData}
                    currentPage={this.props.currentPage}
                    //currentIndex={this.props.currentFramebufferIndex}// TODO
                    onClickLift={this.props.onClick_DmoFramebuffersMenu}
                    //onChangeLift={this.props.onChange_DmoFramebuffersMenu}// TODO
                />

                <DmoQuadScenesPanel
                    currentPage={this.props.currentPage}
                    onClickLift={this.props.onClick_DmoQuadScenesMenu}
                />

                <DmoPolygonScenesPanel
                    currentPage={this.props.currentPage}
                    onClickLift={this.props.onClick_DmoPolygonScenesMenu}
                />

                <DmoImagesPanel
                    currentPage={this.props.currentPage}
                    onClickLift={this.props.onClick_DmoImagesMenu}
                />

                <DmoModelsPanel
                    currentPage={this.props.currentPage}
                    onClickLift={this.props.onClick_DmoModelsMenu}
                />

                <DmoTimelinePanel
                    currentPage={this.props.currentPage}
                    onClickLift={this.props.onClick_DmoTimelineMenu}
                />

                <DmoSyncTracksPanel
                    currentPage={this.props.currentPage}
                    onClickLift={this.props.onClick_DmoSyncTracksMenu}
                />

                <DmoSettingsPanel
                    currentPage={this.props.currentPage}
                    onClickLift={this.props.onClick_DmoSettingsMenu}
                />
            </div>
        );
    }
}
