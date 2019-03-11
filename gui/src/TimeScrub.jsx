// @flow
import React from 'react';
import { Progress, Button, Icon, Level, LevelLeft, LevelItem } from 'bloomer';
import type { ServerMsg, DmoData } from './Helpers';

type TS_Props = {
    value: number,
    max: number,
    onClickLift: (ServerMsg) => void,
};

class TimeSlider extends React.Component<TS_Props> {
    theElement: { current: null | HTMLDivElement }

    constructor(props: TS_Props) {
        super(props);
        this.theElement = React.createRef();
    }

    onClickLocal = (e: SyntheticMouseEvent<>) => {
        if (this.theElement.current !== null) {
            // relative x position of click
            let x = e.nativeEvent.offsetX * (this.props.max / this.theElement.current.offsetWidth);
            let click_value = Number(x.toFixed(2));

            let msg: ServerMsg = {
                data_type: 'SetDmoTime',
                data: JSON.stringify(click_value),
            };
            this.props.onClickLift(msg);
        }
    }

    render() {
        return(
            <div style={{width: "100%"}} ref={this.theElement}>
                <Progress
                    value={this.props.value}
                    max={this.props.max}
                    onClick={this.onClickLocal}
                />
            </div>
        );
    }
}

type TSc_Props = {
    dmoData: ?DmoData,
    currentTime: number,
    onChangeLift: (ServerMsg) => void,
};

export class TimeScrub extends React.Component<TSc_Props> {

    onChangeLocal = (msg: ServerMsg) => {
        this.props.onChangeLift(msg);
    }

    render() {
        let time_max = 100.0;
        if (this.props.dmoData !== null && typeof this.props.dmoData !== 'undefined') {
            time_max = this.props.dmoData.settings.total_length;
        }
        return (
            <div>
            <Level>
                <LevelLeft>
                    <LevelItem>
                        <Button>
                            <Icon className="fas fa-fast-backward fa-lg" />
                        </Button>
                        <Button isColor='success' isOutlined>
                            <Icon className="fas fa-play fa-lg" />
                        </Button>
                        <Button>
                            <Icon className="fas fa-fast-forward fa-lg" />
                        </Button>
                    </LevelItem>
                </LevelLeft>
                <TimeSlider
                    value={this.props.currentTime}
                    max={time_max}
                    onClickLift={this.onChangeLocal}
                />
            </Level>
            </div>
        );
    }
}

