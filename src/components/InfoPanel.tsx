import React, { memo, useMemo } from 'react';
import { SummonerInfo } from '../types';
import { getPhaseDisplayName, getStatusIndicatorClass } from '../utils/gameflow';

interface InfoPanelProps {
  summonerInfo?: SummonerInfo;
  gameflowPhase: string;
}

export const InfoPanel: React.FC<InfoPanelProps> = memo(({ summonerInfo, gameflowPhase }) => {
  const displayName = useMemo(() => {
    return summonerInfo?.display_name || '用户123456';
  }, [summonerInfo?.display_name]);

  const statusInfo = useMemo(() => {
    return {
      className: getStatusIndicatorClass(gameflowPhase),
      displayName: gameflowPhase ? getPhaseDisplayName(gameflowPhase) : "未找到"
    };
  }, [gameflowPhase]);

  return (
    <div className="info-panel">
      <div className="user-name">
        {displayName}
      </div>
      <div className="user-status">
        <span className={`status-indicator ${statusInfo.className}`}></span>
        {statusInfo.displayName}
      </div>
    </div>
  );
});