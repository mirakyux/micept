import { useState, useEffect } from 'react';
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import './App.css';

interface AppState {
  mouse_through: boolean;
  auto_accept: boolean;
  gameflow_phase: string;
  lcu_connected: boolean;
}

function App() {
  const [appState, setAppState] = useState<AppState>({
    mouse_through: true,
    auto_accept: true,
    gameflow_phase: 'None',
    lcu_connected: false
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
      <span 
        className={`phase ${getPhaseClassName(appState.gameflow_phase)}`}  
        data-tauri-drag-region
        title={`自动接受: ${appState.auto_accept ? '开启' : '关闭'} | LCU: ${appState.lcu_connected ? '已连接' : '未连接'}`}
      >
        {getPhaseDisplayName(appState.gameflow_phase)}
        {appState.auto_accept && appState.gameflow_phase === 'ReadyCheck' && (
          <span className="auto-indicator"> 🤖</span>
        )}
      </span>
    </div>
  );
}

export default App;
