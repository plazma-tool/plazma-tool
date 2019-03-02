import React from 'react';
import { Progress, Button, Icon, Level, LevelLeft, LevelItem } from 'bloomer';

// Requires props:
// - value
// - max
// - onClickLift
class TimeSlider extends React.Component {
    constructor(props) {
        super(props);
        this.theElement = React.createRef()
        this.onClickLocal = this.onClickLocal.bind(this);
    }

    onClickLocal(e) {
        // relative x position of click
        let x = e.nativeEvent.offsetX * (this.props.max / this.theElement.current.offsetWidth);
        let click_value = Number(x.toFixed(2));

        let msg = {
            data_type: 'SetDmoTime',
            data: click_value,
        };
        this.props.onClickLift(msg);
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

// Requires props:
// - currentTime
// - totalLength
// - onChangeLift
export class DmoTimeScrub extends React.Component {
    constructor(props) {
        super(props);
        this.onChangeLocal = this.onChangeLocal.bind(this);
    }

    onChangeLocal(msg) {
        this.props.onChangeLift(msg);
    }

    render() {
        let time_max = 100.0;
        if (this.props.dmoData !== null) {
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

