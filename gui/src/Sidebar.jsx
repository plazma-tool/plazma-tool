// @flow
import React from 'react';
//import { Menu, Button, Icon } from 'bloomer';
import type { DmoData } from './Helpers';

import { DmoShadersPanel } from './DmoShaders';
import { DmoFramebuffersPanel } from './DmoFramebuffers';
import { DmoQuadScenesPanel } from './DmoQuadScenes';
import { DmoPolygonScenesPanel } from './DmoPolygonScenes';
import { DmoModelsPanel } from './DmoModels';
import { DmoTimelinePanel } from './DmoTimeline';
import { DmoSyncTracksPanel } from './DmoSyncTracks';
import { DmoSettingsPanel } from './DmoSettings';

type S_Props = {
    dmoData: ?DmoData,
    currentPage: number,
    currentShaderIndex: number,
    onClick_DmoSettingsMenu: () => void,
    onClick_DmoFramebuffersMenu: () => void,
    onClick_DmoQuadScenesMenu: () => void,
    onClick_DmoPolygonScenesMenu: () => void,
    onClick_DmoShadersMenu: () => void,
    onClick_DmoModelsMenu: () => void,
    onClick_DmoTimelineMenu: () => void,
    onClick_DmoSyncTracksMenu: () => void,
    onChange_DmoShadersMenu: (idx: number) => void,
};

export class Sidebar extends React.Component<S_Props> {

    render() {
        if (this.props.dmoData === null || typeof this.props.dmoData === 'undefined') {

            return (
                <div><p>dmoData is empty</p></div>
            );

        } else {

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
}
