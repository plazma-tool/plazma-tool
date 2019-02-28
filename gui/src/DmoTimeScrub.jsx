import React from 'react';
import { Level, LevelLeft, LevelRight, LevelItem, Button, Icon, Progress } from 'bloomer';

// Requires props:
// - currentTime
// - totalLength
// - onChangeLift
export class DmoTimeScrub extends React.Component {
    constructor(props) {
        super(props);
        this.onChangeLocal = this.onChangeLocal.bind(this);
    }

    onChangeLocal(x) {
        this.props.onChangeLift(x);
    }

    render() {
        let time_max = 100.0;
        if (this.props.dmoData !== null) {
            time_max = this.props.dmoData.settings.total_length;
        }
        return (
            <Level onClick={() => this.onChangeLocal("time: " + this.props.time)} >
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
                <Progress value={this.props.currentTime} max={time_max}/>
            </Level>
        );
    }
}

