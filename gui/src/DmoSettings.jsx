import React from 'react';
import { MenuLabel, Field, Label, Control, Input, Checkbox } from 'bloomer';

// TODO Click on the label shows the settings form in the main panel.

export class DmoSettingsMenu extends React.Component {
    render() {
        return (
            <MenuLabel>Settings</MenuLabel>
        );
    }
}

export class DmoSettingsForm extends React.Component {
    render() {
        return (
            <div>

              <Field>
                <Label>Mouse sensitivity</Label>
                <Control>
                  <Input type="number" min="0.0" step="0.1"/>
                </Control>
              </Field>

              <Field>
                <Label>Movement sensitivity</Label>
                <Control>
                  <Input type="number" min="0.0" step="0.1"/>
                </Control>
              </Field>

              <Field>
                <Label>Total length</Label>
                <Control>
                  <Input type="number" min="0.0" step="1.0"/>
                </Control>
              </Field>

              <Field>
                <Control>
                  <Checkbox>Start full screen</Checkbox>
                </Control>
              </Field>

              <Field>
                <Control>
                  <Checkbox>Play audio on start</Checkbox>
                </Control>
              </Field>

            </div>
        );
    }
}
