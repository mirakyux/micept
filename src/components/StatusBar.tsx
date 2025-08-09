import React from 'react';
import { AppState } from '../types';
import { AvatarSection } from './AvatarSection';
import { InfoPanel } from './InfoPanel';

interface StatusBarProps {
  appState: AppState;
}

export const StatusBar: React.FC<StatusBarProps> = ({ appState }) => {
  return (
    <div className="status-bar" data-tauri-drag-region>
      <AvatarSection summonerInfo={appState.summoner_info} />
      <InfoPanel 
        summonerInfo={appState.summoner_info} 
        gameflowPhase={appState.gameflow_phase} 
      />
      {appState.auto_accept && appState.gameflow_phase === 'ReadyCheck' && (
        <span className="auto-indicator">🤖</span>
      )}
    </div>
  );
};