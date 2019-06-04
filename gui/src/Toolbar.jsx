// @flow
import React from 'react';

import logo from './images/idea.svg';

import { ReactComponent as Layout01 } from './images/layout-01.svg';
import { ReactComponent as Layout02 } from './images/layout-02.svg';
import { ReactComponent as Layout03 } from './images/layout-03.svg';
import { ReactComponent as Layout04 } from './images/layout-04.svg';
import { ReactComponent as Layout05 } from './images/layout-05.svg';
import { ReactComponent as Layout06 } from './images/layout-06.svg';
import { ReactComponent as Layout07 } from './images/layout-07.svg';
import { ReactComponent as Layout08 } from './images/layout-08.svg';

import { Input, Title, Box, Modal, ModalBackground, ModalContent, ModalClose, Delete, Field,
    Control, Button, Navbar, NavbarBrand, NavbarItem, Icon, NavbarBurger, NavbarMenu, NavbarStart,
    NavbarEnd, NavbarLink, NavbarDropdown, NavbarDivider } from 'bloomer';

import { EditorsLayout } from './Helpers';
import type { InputEvent, ViewState } from './Helpers';

type OPFM_Props = {
    isActive: bool,
    onClick_Close: () => void,
    onChange_File: (e: InputEvent) => void,
};

class OpenProjectFileModal extends React.Component<OPFM_Props> {
    render()
    {
        return (
            <Modal isActive={this.props.isActive}>
                <ModalBackground />
                <ModalContent>

                    <Box>
                        <Title isSpaced isSize={5} hasTextAlign="centered">
                            Open a Project From File
                            <Delete isPulled="right" onClick={this.props.onClick_Close} />
                        </Title>

                        <Field>
                            <Control>
                                <div className="file is-centered is-boxed">
                                    <label className="file-label">
                                        <Input
                                            className="file-input"
                                            onChange={this.props.onChange_File}
                                            type="file"
                                            name="project_file"
                                        />
                                            <span className="file-cta">
                                                <span className="file-icon">
                                                    <i className="fas fa-upload"></i>
                                                </span>
                                                <span className="file-label">
                                                    Choose a file…
                                                </span>
                                            </span>
                                        </label>
                                    </div>
                            </Control>
                        </Field>
                    </Box>

                </ModalContent>
                <ModalClose onClick={this.props.onClick_Close} />
            </Modal>
        );
    }
}

class OpenPreview extends React.Component<{ isOpen: bool, onClick: () => void, }> {
    render() {
        let color;
        let text;
        if (this.props.isOpen) {
            color = "primary";
            text = "Close Preview";
        } else {
            color = "info";
            text = "Open Preview";
        }

        return (
            <NavbarItem onClick={this.props.onClick}>
                <Button isColor={color} >
                    <Icon className="fas fa-window-maximize" />
                    <span>{text}</span>
                </Button>
            </NavbarItem>
        );
    }
}

type T_Props = {
    isHidden: bool,
    currentLayout: number,
    onClick_SaveProject: () => void,
    onClick_OpenProject: () => void,
    onClick_ReloadProject: () => void,
    onClick_Library: () => void,
    onClick_Preview: () => void,
    onClick_Exit: () => void,
    previewIsOpen: bool,
    view: ViewState,
    onClick_Layout: (layout_index: number) => void,
    onClick_View: (view: ViewState) => void,
};

type T_State= {
    isActive: bool,
    opfm_is_active: bool,
};

export class Toolbar extends React.Component<T_Props, T_State> {
    constructor(props: T_Props)
    {
        super(props);

        this.state = {
            isActive: false,
            opfm_is_active: false,
        };
    }

    onClickNav = () => {}

    render() {
        if (this.props.isHidden) {
            return (<div style={{display: 'none'}}></div>);
        }

        let view_menus = [
            { name: 'time_scrub',   label: "Show Time Scrub (F8)" },
            { name: 'sidebar',      label: "Show Sidebar (F9)" },
            { name: 'toolbar',      label: "Show Toolbar (F10)" },
            { name: 'editors_only', label: "Show Editors Only (F11)" },
        ];

        let view_items = view_menus.map((i) => {
            let icon = "";
            if (this.props.view[i.name]) {
                icon = <Icon className="fas fa-check-square" />;
            } else {
                icon = <Icon className="far fa-square" />;
            }
            let value = this.props.view[i.name];
            return (
                <NavbarItem key={'view_'+i.name+String(value)} onClick={() => {
                    let view = this.props.view;
                    view[i.name] = !view[i.name];
                    this.props.onClick_View(view);
                }}>
                {icon}
                <span>{i.label}</span>
            </NavbarItem>
            );
        });

        return (
            <Navbar>

                <NavbarBrand>
                    <NavbarItem>
                        <img src={logo} alt="logo" style={{ marginRight: 5, width: "50px" }} />
                    </NavbarItem>
                    <NavbarItem isHidden='desktop'>
                        <Icon className='fab fa-github' />
                    </NavbarItem>
                    <NavbarItem isHidden='desktop'>
                        <Icon className='fab fa-twitter' style={{ color: '#55acee' }} />
                    </NavbarItem>
                    <NavbarBurger isActive={this.state.isActive} onClick={this.onClickNav} />
                </NavbarBrand>

                <NavbarMenu isActive={this.state.isActive} onClick={this.onClickNav}>

                    <NavbarStart>

                        <NavbarItem>
                            <Button onClick={this.props.onClick_Library}>
                                <Icon className="fa fa-th-list" />
                                <span>Library</span>
                            </Button>
                        </NavbarItem>

                        <NavbarItem hasDropdown isHoverable>
                            <NavbarLink>
                                <Icon className="fa fa-folder" />
                                <span>File</span>
                            </NavbarLink>
                            <NavbarDropdown>

                                {/*<NavbarItem onClick={() => this.setState({ opfm_is_active: true })}>*/}

                                <NavbarItem onClick={() => { console.log('TODO'); }}>
                                    <Icon className="fa fa-plus-square" />
                                    <span>New...</span>
                                </NavbarItem>

                                <NavbarItem onClick={this.props.onClick_OpenProject}>
                                    <Icon className="fa fa-file-alt" />
                                    <span>Open</span>
                                </NavbarItem>

                                {/*
                                <NavbarItem onClick={() => { console.log('TODO'); }}>
                                    <Icon className="fa fa-file-alt" />
                                    <span>Open Recent</span>
                                </NavbarItem>
                                */}

                                <NavbarItem onClick={this.props.onClick_SaveProject}>
                                    <Icon className="fa fa-save" />
                                    <span>Save</span>
                                </NavbarItem>

                                <NavbarItem onClick={this.props.onClick_ReloadProject}>
                                    <Icon className="fa fa-redo" />
                                    <span>Reload Project From Disk</span>
                                </NavbarItem>

                                {/*
                                <NavbarItem onClick={() => { console.log('TODO'); }}>
                                    <Icon className="fa fa-file-import" />
                                    <span>Import from Shadertoy...</span>
                                </NavbarItem>

                                <NavbarItem onClick={() => { console.log('TODO'); }}>
                                    <Icon className="fa fa-paper-plane" />
                                    <span>Publish on Shadertoy...</span>
                                </NavbarItem>

                                <NavbarItem onClick={() => { console.log('TODO'); }}>
                                    <Icon className="fa fa-cog" />
                                    <span>User Preferences...</span>
                                </NavbarItem>
                                */}

                            </NavbarDropdown>
                        </NavbarItem>

                        <NavbarItem hasDropdown isHoverable>
                            <NavbarLink>
                                <Icon className="fa fa-book-open" />
                                <span>View</span>
                            </NavbarLink>
                            <NavbarDropdown>
                                {view_items}
                            </NavbarDropdown>
                        </NavbarItem>

                        <LayoutNavbarItem
                            currentLayout={this.props.currentLayout}
                            onClickLift={this.props.onClick_Layout}
                        />

                    </NavbarStart>

                    <NavbarEnd>
                        <OpenPreview
                            isOpen={this.props.previewIsOpen}
                            onClick={this.props.onClick_Preview}
                        />

                        <NavbarItem hasDropdown isHoverable>
                            <NavbarLink>
                                <Icon className="fas fa-info-circle" />
                                <span>Help</span>
                            </NavbarLink>
                            <NavbarDropdown>
                                <NavbarItem>One A</NavbarItem>
                                <NavbarItem>Two B</NavbarItem>
                                <NavbarDivider />
                                <NavbarItem>Two A</NavbarItem>
                            </NavbarDropdown>
                        </NavbarItem>

                        <NavbarItem onClick={this.props.onClick_Exit}>
                            <NavbarLink className="is-arrowless">
                                <Icon className="fas fa-times-circle" />
                            </NavbarLink>
                        </NavbarItem>

                    </NavbarEnd>

                </NavbarMenu>

                <OpenProjectFileModal
                    isActive={this.state.opfm_is_active}
                    onChange_File={(e: InputEvent) => {
                        console.log("File name: '" + e.currentTarget.files[0].name + "'");
                        this.setState({ opfm_is_active: false });
                    }}
                    onClick_Close={() => this.setState({ opfm_is_active: false })}
                />
            </Navbar>
        );
    }
}


type LNI_Props = {
    currentLayout: number,
    onClickLift: (layout_index: number) => void,
};

export class LayoutNavbarItem extends React.Component<LNI_Props> {

    render() {

        let fill = '#dee5ed';
        let style = { marginRight: 10 };

        let items_data = [
            {
                label: "One Max",
                comp: <Layout01 width='16px' height='16px' fill={fill} style={style} />,
                idx: EditorsLayout.OneMax,
            },

            {
                label: "Two Vertical",
                comp: <Layout02 width='16px' height='16px' fill={fill} style={style} />,
                idx: EditorsLayout.TwoVertical,
            },

            {
                label: "Two Horizontal",
                comp: <Layout03 width='16px' height='16px' fill={fill} style={style} />,
                idx: EditorsLayout.TwoHorizontal,
            },

            {
                label: "Three, Main Left",
                comp: <Layout04 width='16px' height='16px' fill={fill} style={style} />,
                idx: EditorsLayout.ThreeMainLeft,
            },

            {
                label: "Three, Main Right",
                comp: <Layout05 width='16px' height='16px' fill={fill} style={style} />,
                idx: EditorsLayout.ThreeMainRight,
            },

            {
                label: "Three, Main Top",
                comp: <Layout06 width='16px' height='16px' fill={fill} style={style} />,
                idx: EditorsLayout.ThreeMainTop,
            },

            {
                label: "Three, Main Bottom",
                comp: <Layout07 width='16px' height='16px' fill={fill} style={style} />,
                idx: EditorsLayout.ThreeMainBottom,
            },

            {
                label: "Four Even",
                comp: <Layout08 width='16px' height='16px' fill={fill} style={style} />,
                idx: EditorsLayout.FourEven,
            },
        ];


        let items = items_data.map((i) => {
            return (
                <NavbarItem
                    key={'layout_' + i.idx}
                    onClick={() => this.props.onClickLift(i.idx)}
                >
                    {i.comp}
                    <span>{i.label}</span>
                </NavbarItem>
            );
        });

        let i = items_data[this.props.currentLayout - 1];

        return (
            <NavbarItem hasDropdown isHoverable>
                <NavbarLink>
                    {i.comp}
                    <span>Layout</span>
                </NavbarLink>
                <NavbarDropdown>
                    {items}
                </NavbarDropdown>
            </NavbarItem>
        );
    }
}
