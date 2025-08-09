import React, { memo } from 'react';
import { AppState } from '../types';
import { AvatarSection } from './AvatarSection';
import { InfoPanel } from './InfoPanel';

interface StatusBarProps {
  appState: AppState;
}

export const StatusBar: React.FC<StatusBarProps> = memo(({ appState }) => {
  const showAutoIndicator = appState.auto_accept && appState.gameflow_phase === 'ReadyCheck';
  
  return (
    <div className="status-bar" data-tauri-drag-region>
      <AvatarSection summonerInfo={appState.summoner_info} />
      <InfoPanel 
        summonerInfo={appState.summoner_info} 
        gameflowPhase={appState.gameflow_phase} 
      />
      {showAutoIndicator && (
        <span className="auto-indicator">🤖</span>
      )}
    </div>
  );
});