import { useState, useEffect } from 'react';
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import './App.css';

interface AppState {
  mouse_through: boolean;
  auto_accept: boolean;
  gameflow_phase: string;
  lcu_connected: boolean;
  summoner_info?: {
    display_name: string;
    summoner_level: number;
    profile_icon_id: number;
  };
}

function App() {
  const [appState, setAppState] = useState<AppState>({
    mouse_through: true,
    auto_accept: true,
    gameflow_phase: 'None',
    lcu_connected: false,
    summoner_info: undefined
  });

  // 获取应用状态
  const getAppState = async () => {
    try {
      const state = await invoke<AppState>('get_app_state');
      setAppState(state);
    } catch (err) {
      console.error('获取应用状态失败:', err);
    }
  };

  // 初始化
  useEffect(() => {
    // 获取初始状态
    getAppState();

    // 监听后台状态变化事件
    const unlistenGameflow = listen('gameflow-changed', (event) => {
      console.log('游戏流程状态变化:', event.payload);
      setAppState(prev => ({
        ...prev,
        gameflow_phase: event.payload as string
      }));
    });

    const unlistenMatchAccepted = listen('match-accepted', (event) => {
      console.log('匹配已自动接受:', event.payload);
    });

    // 定期更新状态（降低频率，主要用于同步状态）
    const interval = setInterval(() => {
      getAppState();
    }, 5000);

    return () => {
      clearInterval(interval);
      unlistenGameflow.then(fn => fn());
      unlistenMatchAccepted.then(fn => fn());
    };
  }, []);

  const getPhaseDisplayName = (phase: string) => {
    const phaseMap: { [key: string]: string } = {
      'None': '未连接',
      'Lobby': '大厅',
      'Matchmaking': '匹配中',
      'ReadyCheck': '准备检查',
      'ChampSelect': '英雄选择',
      'InProgress': '游戏中',
      'Reconnect': '重连',
      'WaitingForStats': '等待结算',
      'PreEndOfGame': '游戏结束前',
      'EndOfGame': '游戏结束'
    };
    return phaseMap[phase] || phase;
  };

  const getPhaseClassName = (phase: string) => {
    const classMap: { [key: string]: string } = {
      'ReadyCheck': 'readycheck',
      'Matchmaking': 'matchmaking',
      'ChampSelect': 'champselect',
      'InProgress': 'inprogress'
    };
    return classMap[phase] || '';
  };

  return (
    <div className="app-container">
      <div className="status-bar" data-tauri-drag-region>
        <div className="avatar-section">
          <div className="avatar-container">
            <img 
              src={appState.summoner_info 
                ? `https://ddragon.leagueoflegends.com/cdn/14.1.1/img/profileicon/${appState.summoner_info.profile_icon_id}.png`
                : 'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNTAiIGhlaWdodD0iNTAiIHZpZXdCb3g9IjAgMCA1MCA1MCIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPGNpcmNsZSBjeD0iMjUiIGN5PSIyNSIgcj0iMjUiIGZpbGw9IiMzMzMiLz4KPHN2ZyB4PSIxNSIgeT0iMTAiIHdpZHRoPSIyMCIgaGVpZ2h0PSIzMCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSIjNjY2Ij4KPHA+dGg+TTEyIDEyYzIuMjEgMCA0LTEuNzkgNC00cy0xLjc5LTQtNC00LTQgMS43OS00IDQgMS43OSA0IDQgNHptMCAyYy0yLjY3IDAtOCAxLjM0LTggNHYyaDE2di0yYzAtMi42Ni01LjMzLTQtOC00eiIvPgo8L3N2Zz4KPC9zdmc+'
              }
              alt="头像"
              className="summoner-avatar"
            />
            <div className="avatar-level">
              {appState.summoner_info ? appState.summoner_info.summoner_level : '等级'}
            </div>
          </div>
        </div>
        <div className="info-panel">
          <div className="user-name">
            {appState.summoner_info ? appState.summoner_info.display_name : '用户123456'}
          </div>
          <div className="user-status">
            <span className="status-indicator"></span>
            {appState.summoner_info 
              ? '游戏中' 
              : getPhaseDisplayName(appState.gameflow_phase)
            }
          </div>
        </div>
        {appState.auto_accept && appState.gameflow_phase === 'ReadyCheck' && (
          <span className="auto-indicator">🤖</span>
        )}
      </div>
    </div>
  );
}

export default App;
