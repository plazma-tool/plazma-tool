// @flow
import React from 'react';

import logo from './idea.svg';

import { Input, Title, Box, Modal, ModalBackground, ModalContent, ModalClose, ModalCard, ModalCardHeader, ModalCardBody, ModalCardTitle, ModalCardFooter, Delete, Field, Control, Button, Navbar, NavbarBrand, NavbarItem, Icon, NavbarBurger, NavbarMenu,
    NavbarStart, NavbarEnd, NavbarLink, NavbarDropdown, NavbarDivider } from 'bloomer';

import type { InputEvent } from './Helpers';

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
                <Button isColor={color} >{text}</Button>
            </NavbarItem>
        );
    }
}

type T_Props = {
    onClick_Library: () => void,
    onClick_Preview: () => void,
    onClick_Exit: () => void,
    previewIsOpen: bool,
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

    render()
    {
        return (
            <Navbar style={{ marginBottom: '10px' }}>

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
                            <Field isGrouped>
                                <Control>
                                    <Button onClick={this.props.onClick_Library}>
                                        <Icon className="fa fa-th-list" />
                                        {/* <span>Library</span> */}
                                    </Button>
                                </Control>
                            </Field>
                        </NavbarItem>

                        <NavbarItem hasDropdown isHoverable>
                            <NavbarLink>
                                <Icon className="fa fa-folder-open" />
                            </NavbarLink>
                            <NavbarDropdown>

                                <NavbarItem onClick={() => this.setState({ opfm_is_active: true })}>
                                    Open Project from File ...
                                </NavbarItem>

                                {/* <NavbarItem>Import from Shadertoy ...</NavbarItem> */}

                            </NavbarDropdown>
                        </NavbarItem>

                        <NavbarItem hasDropdown isHoverable>
                            <NavbarLink>
                                <Icon className="fa fa-save" />
                            </NavbarLink>
                            <NavbarDropdown>
                                <NavbarItem>
                                    <Icon className="fa fa-download" />
                                    <span>Save as File ...</span>
                                </NavbarItem>
                                <NavbarItem>
                                    <Icon className="fa fa-cloud-upload-alt" />
                                    <span>Publish on Shadertoy ...</span>
                                </NavbarItem>
                            </NavbarDropdown>
                        </NavbarItem>

                    </NavbarStart>

                    <NavbarEnd>
                        <OpenPreview
                            isOpen={this.props.previewIsOpen}
                            onClick={this.props.onClick_Preview}
                        />

                        <NavbarItem hasDropdown isHoverable>
                            <NavbarLink>Info</NavbarLink>
                            <NavbarDropdown>
                                <NavbarItem>One A</NavbarItem>
                                <NavbarItem>Two B</NavbarItem>
                                <NavbarDivider />
                                <NavbarItem>Two A</NavbarItem>
                            </NavbarDropdown>
                        </NavbarItem>

                        <NavbarItem onClick={this.props.onClick_Exit}>
                            <Button> <Delete /> </Button>
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
