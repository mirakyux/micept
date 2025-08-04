import { useState, useEffect } from 'react';
import { invoke } from "@tauri-apps/api/core";
import './App.css';

interface LcuAuthInfo {
  port: string;
  token: string;
  is_connected: boolean;
}

interface SummonerInfo {
  display_name: string;
  summoner_level: number;
  profile_icon_id: number;
}

interface GameflowSession {
  phase: string;
}

interface AdminStatus {
  is_admin: boolean;
  message: string;
}

function App() {
  const [lcuAuth, setLcuAuth] = useState<LcuAuthInfo | null>(null);
  const [summonerInfo, setSummonerInfo] = useState<SummonerInfo | null>(null);
  const [gameflowPhase, setGameflowPhase] = useState<string>('None');
  const [adminStatus, setAdminStatus] = useState<AdminStatus | null>(null);
  const [autoAccept, setAutoAccept] = useState<boolean>(false);
  const [error, setError] = useState<string>('');
  const [isRefreshing, setIsRefreshing] = useState<boolean>(false);

  // 检查管理员权限
  const checkAdminPrivileges = async () => {
    try {
      const status = await invoke<AdminStatus>('check_admin_privileges');
      setAdminStatus(status);
    } catch (err) {
      console.error('检查管理员权限失败:', err);
      setAdminStatus({
        is_admin: false,
        message: '无法检查管理员权限'
      });
    }
  };

  // 获取LCU认证信息
  const getLcuAuth = async () => {
    try {
      const auth = await invoke<LcuAuthInfo>('get_lcu_auth');
      setLcuAuth(auth);
      setError('');
      return auth;
    } catch (err) {
      setError(err as string);
      setLcuAuth(null);
      setSummonerInfo(null);
      return null;
    }
  };

  // 获取召唤师信息
  const getSummonerInfo = async (auth: LcuAuthInfo) => {
    try {
      const info = await invoke<SummonerInfo>('get_summoner_info', {
        port: auth.port,
        token: auth.token
      });
      setSummonerInfo(info);
    } catch (err) {
      console.error('获取召唤师信息失败:', err);
      setSummonerInfo(null);
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
      if (autoAccept && session.phase === 'ReadyCheck') {
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

  // 刷新所有信息
  const refreshAll = async () => {
    setIsRefreshing(true);
    const auth = await getLcuAuth();
    if (auth) {
      await Promise.all([
        getSummonerInfo(auth),
        getGameflowPhase(auth)
      ]);
    }
    setIsRefreshing(false);
  };

  // 手动接受匹配
  const handleManualAccept = async () => {
    if (lcuAuth) {
      await acceptMatch(lcuAuth);
    }
  };

  // 初始化和定时刷新
  useEffect(() => {
    checkAdminPrivileges();
    refreshAll();
    
    const interval = setInterval(() => {
      if (lcuAuth) {
        getGameflowPhase(lcuAuth);
      } else {
        getLcuAuth();
      }
    }, 2000);

    return () => clearInterval(interval);
  }, [autoAccept]);

  const getPhaseDisplayName = (phase: string) => {
    const phaseMap: { [key: string]: string } = {
      'None': '无',
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
      {/* 头部 */}
      <div className="header">
        <h1>英雄联盟自动接受助手</h1>
        
        {/* 管理员权限状态 */}
        {adminStatus && (
          <div className={`connection-status ${adminStatus.is_admin ? 'connected' : 'disconnected'}`}>
            <div className="status-indicator"></div>
            <span>{adminStatus.message}</span>
          </div>
        )}
        
        {/* LCU连接状态 */}
        <div className={`connection-status ${lcuAuth ? 'connected' : 'disconnected'}`}>
          <div className="status-indicator"></div>
          <span>{lcuAuth ? 'LCU已连接' : 'LCU未连接'}</span>
        </div>
      </div>

      {/* 错误信息 */}
      {error && (
        <div className="error-message">
          {error}
        </div>
      )}

      {/* 召唤师信息 */}
      {summonerInfo && (
        <div className="summoner-card">
          <div className="summoner-avatar">
            <img 
              src={`https://ddragon.leagueoflegends.com/cdn/13.24.1/img/profileicon/${summonerInfo.profile_icon_id}.png`}
              alt="召唤师头像"
              onError={(e) => {
                (e.target as HTMLImageElement).src = 'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iODAiIGhlaWdodD0iODAiIHZpZXdCb3g9IjAgMCA4MCA4MCIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPGNpcmNsZSBjeD0iNDAiIGN5PSI0MCIgcj0iNDAiIGZpbGw9IiNjODliM2MiLz4KPHN2ZyB4PSIyMCIgeT0iMjAiIHdpZHRoPSI0MCIgaGVpZ2h0PSI0MCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSIjNDYzNzE0Ij4KPHA+VXNlcjwvcD4KPC9zdmc+Cjwvc3ZnPgo=';
              }}
            />
          </div>
          <div className="summoner-info">
            <h2>{summonerInfo.display_name}</h2>
            <p>等级: {summonerInfo.summoner_level}</p>
          </div>
        </div>
      )}

      {/* 游戏状态 */}
      <div className="game-status">
        <h3>游戏状态</h3>
        <div className="status-item">
          <span className="label">当前阶段:</span>
          <span className={`phase ${getPhaseClassName(gameflowPhase)}`}>
            {getPhaseDisplayName(gameflowPhase)}
          </span>
        </div>
      </div>

      {/* 控制面板 */}
      <div className="controls">
        {/* 自动接受开关 */}
        <div className="auto-accept-toggle">
          <label className="toggle-switch">
            <input
              type="checkbox"
              checked={autoAccept}
              onChange={(e) => setAutoAccept(e.target.checked)}
              disabled={!lcuAuth}
            />
            <span className="slider"></span>
          </label>
          <span className="toggle-label">
            自动接受匹配 {autoAccept ? '(已开启)' : '(已关闭)'}
          </span>
        </div>

        {/* 操作按钮 */}
        <button
          className="accept-button"
          onClick={handleManualAccept}
          disabled={!lcuAuth || gameflowPhase !== 'ReadyCheck'}
        >
          手动接受匹配
        </button>

        <button
          className="refresh-button"
          onClick={refreshAll}
          disabled={isRefreshing}
        >
          {isRefreshing ? '刷新中...' : '刷新状态'}
        </button>
      </div>

      {/* 页脚 */}
      <div className="footer">
        <p>英雄联盟自动接受助手</p>
        <p>请确保以管理员权限运行以获得最佳体验</p>
        <p>自动检测游戏状态，智能接受匹配</p>
      </div>
    </div>
  );
}

export default App;
