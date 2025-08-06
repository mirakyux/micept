import { useState, useEffect } from 'react';
import { invoke } from "@tauri-apps/api/core";
import './App.css';

interface LcuAuthInfo {
  port: string;
  token: string;
  is_connected: boolean;
}

interface GameflowSession {
  phase: string;
}

function App() {
  const [lcuAuth, setLcuAuth] = useState<LcuAuthInfo | null>(null);
  const [gameflowPhase, setGameflowPhase] = useState<string>('None');
  const [autoRefresh] = useState<boolean>(true);



  // 获取LCU认证信息
  const getLcuAuth = async () => {
    try {
      const auth = await invoke<LcuAuthInfo>('get_lcu_auth');
      setLcuAuth(auth);
      return auth;
    } catch (err) {
      setLcuAuth(null);
      return null;
    }
  };



  // 获取游戏流程状态
  const getGameflowPhase = async (auth: LcuAuthInfo) => {
    try {
      const session = await invoke<GameflowSession>('get_gameflow_phase', {
        port: auth.port,
        token: auth.token
      });
      setGameflowPhase(session.phase);
      
      // 自动接受匹配
      if (session.phase === 'ReadyCheck') {
        await acceptMatch(auth);
      }
    } catch (err) {
      console.error('获取游戏流程状态失败:', err);
      setGameflowPhase('None');
    }
  };
  // 接受匹配
  const acceptMatch = async (auth: LcuAuthInfo) => {
    try {
      await invoke<string>('accept_match', {
        port: auth.port,
        token: auth.token
      });
      console.log('匹配已接受');
    } catch (err) {
      console.error('接受匹配失败:', err);
    }
  };

  // 初始化和定时刷新
  useEffect(() => {
    
    let interval: number | undefined;
    
    interval = setInterval(() => {
      if (lcuAuth) {
        getGameflowPhase(lcuAuth);
      } else {
        getLcuAuth();
      }
    }, 2000);

    return () => {
      if (interval) clearInterval(interval);
    };
  }, [autoRefresh, lcuAuth]);



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
      <span className={`phase ${getPhaseClassName(gameflowPhase)}`}>
        {getPhaseDisplayName(gameflowPhase)}
      </span>
    </div>
  );
}

export default App;
