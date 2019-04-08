// @flow
import React from 'react';

import { Column, Columns, Title, Content, Card, CardHeader, CardHeaderTitle, CardHeaderIcon,
    CardImage, CardContent, Icon, Media, MediaContent, MediaLeft, Image, Subtitle } from 'bloomer';

//import { CurrentPage } from './Helpers';
import type { ServerMsg, DmoData, InputEvent } from './Helpers';

type LP_Props = {
    dmoData: DmoData,
    onChangeLift: (ServerMsg) => void,
};

export class LibraryPage extends React.Component<LP_Props> {

    onChangeLocal = (e: InputEvent) => {
        let msg = {
            data_type: 'TODO: compose the message',
            data: '',
        };
        this.props.onChangeLift(msg);
    }

    render() {
        return (
            <div>
                <Columns>

                    <Column isSize="1/3">
                        <Card>
                            <CardHeader>
                                <CardHeaderTitle>
                                    Component
                                </CardHeaderTitle>
                                <CardHeaderIcon>
                                    <Icon className="fa fa-angle-down" />
                                </CardHeaderIcon>
                            </CardHeader>
                            <CardImage>
                                <Image isRatio='4:3' src='https://via.placeholder.com/1280x960' />
                            </CardImage>
                            <CardContent>
                                <Media>
                                    <MediaLeft>
                                        <Image isSize='48x48' src='https://via.placeholder.com/96x96' />
                                    </MediaLeft>
                                    <MediaContent>
                                        <Title isSize={4}>John Wick</Title>
                                        <Subtitle isSize={6}>@John Wick</Subtitle>
                                    </MediaContent>
                                </Media>
                                <Content>
                                    People Keep Asking If I’m Back, And I Haven’t Really Had An Answer, But Now, Yeah, I’m Thinking I’m Back.
                                    <br/>
                                    <small>11:09 PM - 30 October 2014</small>
                                </Content>
                            </CardContent>
                        </Card>
                    </Column>

                    <Column isSize="1/3">
                        <Card>
                            <CardHeader>
                                <CardHeaderTitle>
                                    Component
                                </CardHeaderTitle>
                                <CardHeaderIcon>
                                    <Icon className="fa fa-angle-down" />
                                </CardHeaderIcon>
                            </CardHeader>
                            <CardImage>
                                <Image isRatio='4:3' src='https://via.placeholder.com/1280x960' />
                            </CardImage>
                            <CardContent>
                                <Media>
                                    <MediaLeft>
                                        <Image isSize='48x48' src='https://via.placeholder.com/96x96' />
                                    </MediaLeft>
                                    <MediaContent>
                                        <Title isSize={4}>John Wick</Title>
                                        <Subtitle isSize={6}>@John Wick</Subtitle>
                                    </MediaContent>
                                </Media>
                                <Content>
                                    People Keep Asking If I’m Back, And I Haven’t Really Had An Answer, But Now, Yeah, I’m Thinking I’m Back.
                                    <br/>
                                    <small>11:09 PM - 30 October 2014</small>
                                </Content>
                            </CardContent>
                        </Card>
                    </Column>

                    <Column isSize="1/3">
                        <Card>
                            <CardHeader>
                                <CardHeaderTitle>
                                    Component
                                </CardHeaderTitle>
                                <CardHeaderIcon>
                                    <Icon className="fa fa-angle-down" />
                                </CardHeaderIcon>
                            </CardHeader>
                            <CardImage>
                                <Image isRatio='4:3' src='https://via.placeholder.com/1280x960' />
                            </CardImage>
                            <CardContent>
                                <Media>
                                    <MediaLeft>
                                        <Image isSize='48x48' src='https://via.placeholder.com/96x96' />
                                    </MediaLeft>
                                    <MediaContent>
                                        <Title isSize={4}>John Wick</Title>
                                        <Subtitle isSize={6}>@John Wick</Subtitle>
                                    </MediaContent>
                                </Media>
                                <Content>
                                    People Keep Asking If I’m Back, And I Haven’t Really Had An Answer, But Now, Yeah, I’m Thinking I’m Back.
                                    <br/>
                                    <small>11:09 PM - 30 October 2014</small>
                                </Content>
                            </CardContent>
                        </Card>
                    </Column>

                </Columns>

            </div>
        );
    }
}
