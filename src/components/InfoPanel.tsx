import React from 'react';
import { SummonerInfo } from '../types';
import { getPhaseDisplayName, getStatusIndicatorClass } from '../utils/gameflow';

interface InfoPanelProps {
  summonerInfo?: SummonerInfo;
  gameflowPhase: string;
}

export const InfoPanel: React.FC<InfoPanelProps> = ({ summonerInfo, gameflowPhase }) => {
  return (
    <div className="info-panel">
      <div className="user-name">
        {summonerInfo && summonerInfo.display_name 
          ? summonerInfo.display_name 
          : '用户123456'}
      </div>
      <div className="user-status">
        <span className={`status-indicator ${getStatusIndicatorClass(gameflowPhase)}`}></span>
        {gameflowPhase 
          ? getPhaseDisplayName(gameflowPhase) : "未找到"
        }
      </div>
    </div>
  );
};