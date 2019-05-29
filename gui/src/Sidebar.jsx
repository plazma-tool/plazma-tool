// @flow
import React from 'react';
import type { DmoData, Shader } from './Helpers';

import { DmoShadersPanel } from './DmoShaders';
import { DmoPropertiesPanel } from './DmoProperties';
import { DmoDataPanel } from './DmoData';

type S_Props = {
    dmoData: ?DmoData,
    shaders: Shader[],
    currentPage: number,
    currentShaderIndex: number,
    onClick_DmoShadersMenu: () => void,
    onClick_DmoDataMenu: () => void,
    onClick_DmoPropertiesMenu: () => void,
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
                <DmoPropertiesPanel
                    currentPage={this.props.currentPage}
                    onClickLift={this.props.onClick_DmoPropertiesMenu}
                />

                <DmoShadersPanel
                    dmoData={this.props.dmoData}
                    shaders={this.props.shaders}
                    currentPage={this.props.currentPage}
                    currentIndex={this.props.currentShaderIndex}
                    onClickLift={this.props.onClick_DmoShadersMenu}
                    onChangeLift={this.props.onChange_DmoShadersMenu}
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
