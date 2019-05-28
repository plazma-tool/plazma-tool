// @flow
import React from 'react';
//import { Menu, Button, Icon } from 'bloomer';
import type { DmoData, Shader } from './Helpers';

import { DmoShadersPanel } from './DmoShaders';
import { DmoSettingsPanel } from './DmoSettings';
import { DmoDataPanel } from './DmoData';

type S_Props = {
    dmoData: ?DmoData,
    shaders: Shader[],
    currentPage: number,
    currentShaderIndex: number,
    onClick_DmoShadersMenu: () => void,
    onClick_DmoSettingsMenu: () => void,
    onClick_DmoDataMenu: () => void,
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
                    shaders={this.props.shaders}
                    currentPage={this.props.currentPage}
                    currentIndex={this.props.currentShaderIndex}
                    onClickLift={this.props.onClick_DmoShadersMenu}
                    onChangeLift={this.props.onChange_DmoShadersMenu}
                />

                <DmoSettingsPanel
                    currentPage={this.props.currentPage}
                    onClickLift={this.props.onClick_DmoSettingsMenu}
                />

                <DmoDataPanel
                    currentPage={this.props.currentPage}
                    onClickLift={this.props.onClick_DmoDataMenu}
                />
            </div>
        );
        }
    }
}
