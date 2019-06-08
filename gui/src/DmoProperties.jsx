// @flow
import React from 'react';
import { Column, Columns, Panel, PanelHeading, Field, Label, Control, Input, Checkbox, Title } from 'bloomer';
import { CurrentPage } from './Helpers';
import type { ServerMsg, DmoData, InputEvent } from './Helpers';

type DPP_Props = {
    currentPage: number,
    onClickLift: () => void,
};

export class DmoPropertiesPanel extends React.Component<DPP_Props> {
    render() {
        let color = "";
        if (this.props.currentPage === CurrentPage.Properties) {
            color = "primary";
        }
        return (
            <Panel onClick={this.props.onClickLift}>
                <PanelHeading hasTextColor={color}>Properties</PanelHeading>
            </Panel>
        );
    }
}

type SSec_Props = {
    dmoData: DmoData,
    onChangeLift: (ServerMsg) => void,
};

class SettingsSection extends React.Component<SSec_Props> {

    onChangeLocal = (e: InputEvent) => {
        let s = this.props.dmoData.settings;
        switch(e.currentTarget.type) {
            case 'number':
                s[e.currentTarget.name] = Number(e.currentTarget.value);
                break;
            case 'checkbox':
                s[e.currentTarget.name] = e.currentTarget.checked;
                break;
            default:
                s[e.currentTarget.name] = e.currentTarget.value;
        }

        let msg: ServerMsg = {
            data_type: 'SetSettings',
            data: JSON.stringify(s),
        };
        this.props.onChangeLift(msg);
    }

    render() {
        return (
            <Columns isMultiline={true}>
                <Column isSize='full'>
                    <Title>Settings</Title>
                </Column>
                <Column isSize='1/2'>

                    <Field>
                        <Label>Mouse sensitivity</Label>
                        <Control>
                            <Input
                                name='mouse_sensitivity'
                                value={this.props.dmoData.settings.mouse_sensitivity}
                                type="number" min="0.0" step="0.1"
                                onChange={this.onChangeLocal}
                            />
                        </Control>
                    </Field>

                    <Field>
                        <Label>Movement sensitivity</Label>
                        <Control>
                            <Input
                                name='movement_sensitivity'
                                value={this.props.dmoData.settings.movement_sensitivity}
                                type="number" min="0.0" step="0.1"
                                onChange={this.onChangeLocal}
                            />
                        </Control>
                    </Field>

                </Column>
                <Column isSize='1/2'>

                    <Field>
                        <Label>Total length</Label>
                        <Control>
                            <Input
                                name='total_length'
                                value={this.props.dmoData.settings.total_length}
                                type="number" min="0.0" step="1.0"
                                onChange={this.onChangeLocal}
                            />
                        </Control>
                    </Field>

                    <Field>
                        <Control>
                            <Checkbox
                                name='start_full_screen'
                                checked={this.props.dmoData.settings.start_full_screen}
                                onChange={this.onChangeLocal}
                            >
                                Start full screen
                            </Checkbox>
                        </Control>
                    </Field>

                    <Field>
                        <Control>
                            <Checkbox
                                name='audio_play_on_start'
                                checked={this.props.dmoData.settings.audio_play_on_start}
                                onChange={this.onChangeLocal}
                            >
                                Play audio on start
                            </Checkbox>
                        </Control>
                    </Field>

                </Column>
            </Columns>
        );
    }
}

type MSec_Props = {
    dmoData: DmoData,
    onChangeLift: (ServerMsg) => void,
};

class MetadataSection extends React.Component<MSec_Props> {

    onChangeLocal = (e: InputEvent) => {
        let m = this.props.dmoData.metadata;
        switch(e.currentTarget.type) {
            case 'number':
                m[e.currentTarget.name] = Number(e.currentTarget.value);
                break;
            case 'checkbox':
                m[e.currentTarget.name] = e.currentTarget.checked;
                break;
            default:
                m[e.currentTarget.name] = e.currentTarget.value;
        }

        let msg: ServerMsg = {
            data_type: 'SetMetadata',
            data: JSON.stringify(m),
        };
        this.props.onChangeLift(msg);
    }

    render() {
        return (
            <Columns isMultiline={true}>
                <Column isSize='full'>
                    <Title>Metadata</Title>
                </Column>

                <Column isSize='1/2'>
                    <Field>
                        <Label>Title</Label>
                        <Control>
                            <Input
                                name='title'
                                value={this.props.dmoData.metadata.title}
                                type='text'
                                onChange={this.onChangeLocal}
                            />
                        </Control>
                    </Field>

                    <Field>
                        <Label>Tags</Label>
                        <Control>
                            <Input
                                name='tags'
                                value={this.props.dmoData.metadata.tags}
                                type='text'
                                onChange={this.onChangeLocal}
                            />
                        </Control>
                    </Field>

                    <Field>
                        <Label>Description</Label>
                        <Control>
                            <Input
                                name='description'
                                value={this.props.dmoData.metadata.description}
                                type='text'
                                onChange={this.onChangeLocal}
                            />
                        </Control>
                    </Field>

                </Column>
                <Column isSize='1/2'>

                    <Field>
                        <Label>Author</Label>
                        <Control>
                            <Input
                                name='author'
                                value={this.props.dmoData.metadata.author}
                                type='text'
                                onChange={this.onChangeLocal}
                            />
                        </Control>
                    </Field>

                    <Field>
                        <Label>Url</Label>
                        <Control>
                            <Input
                                name='url'
                                value={this.props.dmoData.metadata.url}
                                type='text'
                                onChange={this.onChangeLocal}
                            />
                        </Control>
                    </Field>

                    <Field>
                        <Label>Created</Label>
                        <Control>
                            <Input
                                name='created'
                                value={this.props.dmoData.metadata.created}
                                type='text'
                                onChange={this.onChangeLocal}
                            />
                        </Control>
                    </Field>

                    <Field>
                        <Label>Updated</Label>
                        <Control>
                            <Input
                                name='updated'
                                value={this.props.dmoData.metadata.updated}
                                type='text'
                                onChange={this.onChangeLocal}
                            />
                        </Control>
                    </Field>

                </Column>

            </Columns>
        );
    }
}

type PP_Props = {
    dmoData: DmoData,
    onChange_Metadata: (ServerMsg) => void,
    onChange_Settings: (ServerMsg) => void,
};

export class PropertiesPage extends React.Component<PP_Props> {
    render() {
        return (
            <div>
                <MetadataSection
                    dmoData={this.props.dmoData}
                    onChangeLift={this.props.onChange_Metadata}
                />

                <SettingsSection
                    dmoData={this.props.dmoData}
                    onChangeLift={this.props.onChange_Settings}
                />
            </div>
        );
    }
}

