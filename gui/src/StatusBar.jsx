// @flow
import React from 'react';
import { TextArea } from 'bloomer';
import type { EditorErrorData } from './Helpers';

type SB_Props = {
    editorContent: string,
    editorErrorData: ?EditorErrorData,
};

export class StatusBar extends React.Component<SB_Props> {
    render() {
        let content = "";

        if (this.props.editorErrorData !== null && typeof this.props.editorErrorData !== 'undefined') {
            let text = this.props.editorErrorData.text;
            if (text.trim().length > 0) {
                content = text;
            } else {
                content = this.props.editorContent;
            }
        } else {
            content = this.props.editorContent;
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

