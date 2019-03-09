import React from 'react';

import logo from './idea.svg';

import { Field, Control, Button, Navbar, NavbarBrand, NavbarItem, Icon, NavbarBurger, NavbarMenu,
    NavbarStart, NavbarEnd, NavbarLink, NavbarDropdown, NavbarDivider } from 'bloomer';

export class Toolbar extends React.Component {
    constructor(props)
    {
        super(props);

        this.state = {
            isActive: false,
        };
    }

    render()
    {
        return (
            <Navbar style={{ borderBottom: 'solid 1px #00D1B2', margin: '0' }}>

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
                                    <Button>
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
                                <NavbarItem>Open File ...</NavbarItem>
                                <NavbarItem>Import from Shadertoy ...</NavbarItem>
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
                        <NavbarItem hasDropdown isHoverable>
                            <NavbarLink>Info</NavbarLink>
                            <NavbarDropdown>
                                <NavbarItem>One A</NavbarItem>
                                <NavbarItem>Two B</NavbarItem>
                                <NavbarDivider />
                                <NavbarItem>Two A</NavbarItem>
                            </NavbarDropdown>
                        </NavbarItem>
                        <NavbarItem>
                            <Icon className='fa fa-times' style={{ color: '#55acee' }} />
                        </NavbarItem>
                    </NavbarEnd>

                </NavbarMenu>
            </Navbar>
        );
    }
}
