import React, { memo, useMemo } from 'react';
import { SummonerInfo } from '../types';
import { calculateXpProgress } from '../utils/gameflow';

interface AvatarSectionProps {
  summonerInfo?: SummonerInfo;
}

export const AvatarSection: React.FC<AvatarSectionProps> = memo(({ summonerInfo }) => {
  const xpProgress = useMemo(() => {
    return summonerInfo 
      ? calculateXpProgress(summonerInfo.xp_since_last_level, summonerInfo.xp_until_next_level)
      : 0;
  }, [summonerInfo?.xp_since_last_level, summonerInfo?.xp_until_next_level]);

  const avatarUrl = useMemo(() => {
    return summonerInfo 
      ? `https://ddragon.leagueoflegends.com/cdn/14.1.1/img/profileicon/${summonerInfo.profile_icon_id}.png`
      : '/icon.png';
  }, [summonerInfo?.profile_icon_id]);

  const strokeDashoffset = useMemo(() => {
    const circumference = 2 * Math.PI * 30;
    return circumference * (1 - xpProgress / 100);
  }, [xpProgress]);

  return (
    <div className="avatar-section">
      <div className="avatar-container">
        <svg className="xp-progress-ring" width="64" height="64">
          <circle
            className="xp-progress-bg"
            cx="32"
            cy="32"
            r="30"
            fill="none"
            stroke="rgba(201, 170, 113, 0.3)"
            strokeWidth="2"
          />
          <circle
            className="xp-progress-fill"
            cx="32"
            cy="32"
            r="30"
            fill="none"
            stroke="#c9aa71"
            strokeWidth="2"
            strokeLinecap="round"
            strokeDasharray="188.5"
            strokeDashoffset={strokeDashoffset}
            transform="rotate(-90 32 32)"
          />
        </svg>
        <img 
          src={avatarUrl}
          alt="头像"
          className="summoner-avatar"
        />
        <div className="avatar-level">
          {summonerInfo ? summonerInfo.summoner_level : '等级'}
        </div>
      </div>
    </div>
  );
});