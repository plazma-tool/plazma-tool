import React from 'react';
import { Title, Field, Label, Control, Input } from 'bloomer';
import { CurrentPage } from './Helpers';

// Requires props:
// - currentPage
// - onClickLift
export class DmoModelsMenu extends React.Component {
    render() {
        let color = "";
        if (this.props.currentPage === CurrentPage.Models) {
            color = "primary";
        }

        return (
            <div onClick={this.props.onClickLift}>
                <Title tag='h1' hasTextColor={color}>Models</Title>
            </div>
        );
    }
}

// Requires props:
// - dmoData
// - onChangeLift
export class ModelsPage extends React.Component {
    constructor(props) {
        super(props);
        this.onChangeLocal = this.onChangeLocal.bind(this);
    }

    onChangeLocal(e) {
        let data = {};
        let msg = {
            data_type: 'TODO: compose the message',
            data: data,
        };
        this.props.onChangeLift(msg);
    }

    render() {
        return (
            <div>

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

            </div>
        );
    }
}
