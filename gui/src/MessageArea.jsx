// @flow
import React from 'react';
import { TextArea } from 'bloomer';
import type { Shader } from './Helpers';

type MA_Props = {
    shader: Shader,
};

export class MessageArea extends React.Component<MA_Props> {
    render() {
        let content = "";
        let shader = this.props.shader;

        if (shader.error_data !== null && typeof shader.error_data !== 'undefined') {
            let text = shader.error_data.text;
            if (text.trim().length > 0) {
                content = text;
            }
        }

        return (
            <TextArea
                className="status-bar"
                value={content}
                readOnly
            />
        );
    }
}

