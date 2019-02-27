import React from 'react';
import { MenuLabel, MenuList, MenuLink } from 'bloomer';

// TODO Use a collapsed and expanded state. Click on the menu label expands a
// tree. Selecting a shader opens it in the editor.

function getShaderIndex(dmoData, selectedPath) {
    if (dmoData === null) {
        return 0;
    }
    let idx = dmoData.context.index.shader_path_to_idx[selectedPath];
    if (idx === null) {
        console.log("Error: selectedPath not found in shaders");
        return 0;
    }
    let n = Number(idx);
    if (!isNaN(n)) {
        return n;
    } else {
        console.log("Error, index is not a number");
        return 0;
    }
}

function pathBasename(path) {
    return path.replace(/.*\//, '')
}

// Requires props:
// - dmoData
// - currentIndex
// - onChangeLift
export class DmoContextMenu extends React.Component {
    constructor(props) {
        super(props);
        this.onChangeLocal = this.onChangeLocal.bind(this);
    }

    onChangeLocal(currentIndex) {
        this.props.onChangeLift(currentIndex);
    }

    render() {
        let paths = [];
        if (this.props.dmoData !== null) {
            paths = Object.entries(this.props.dmoData.context.index.shader_path_to_idx);
        };

        let pathLinks = paths.map((i) => {
            let path_full = i[0];
            let path_index = i[1];
            let link;

            if (path_index === this.props.currentIndex) {
                link = <MenuLink isActive>{pathBasename(path_full)}</MenuLink>
            } else {
                link = <MenuLink>{pathBasename(path_full)}</MenuLink>
            };
            return (
                <li
                    key={path_full}
                    onClick={() => this.onChangeLocal(path_index)}
                >
                    {link}
                </li>
            );
        });

        return (
            <div>
                <MenuLabel>Context</MenuLabel>
                <MenuLabel>Shaders</MenuLabel>
                <MenuList>
                    {pathLinks}
                </MenuList>
            </div>
        );
    }
}
