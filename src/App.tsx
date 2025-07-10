import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface SummonerInfo {
  display_name: string;
  summoner_level: number;
  profile_icon_id: number;
}

interface LcuAuthInfo {
  port: number | null;
  token: string | null;
  base_url: string | null;
}

function App() {
  const [lcuConnected, setLcuConnected] = useState(false);
  const [summonerInfo, setSummonerInfo] = useState<SummonerInfo | null>(null);
  const [gameflowPhase, setGameflowPhase] = useState<string>("");
  const [autoAcceptEnabled, setAutoAcceptEnabled] = useState(false);
  const [status, setStatus] = useState<string>("检查LCU连接中...");
  const [error, setError] = useState<string>("");

  // 检查LCU连接状态
  const checkLcuConnection = async () => {
    try {
      const authInfo: LcuAuthInfo = await invoke("get_lcu_auth");
      if (authInfo.port && authInfo.token) {
        setLcuConnected(true);
        setStatus("LCU已连接");
        setError("");
        return true;
      } else {
        setLcuConnected(false);
        setStatus("LCU未运行");
        setSummonerInfo(null);
        return false;
      }
    } catch (err) {
      setLcuConnected(false);
      setStatus("LCU连接失败");
      setError(err as string);
      return false;
    }
  };

  // 获取召唤师信息
  const fetchSummonerInfo = async () => {
    try {
      const info: SummonerInfo = await invoke("get_summoner_info");
      setSummonerInfo(info);
      setError("");
    } catch (err) {
      setError(`获取召唤师信息失败: ${err}`);
      setSummonerInfo(null);
    }
  };

  // 获取游戏流程状态
  const fetchGameflowPhase = async () => {
    try {
      const phase: string = await invoke("get_gameflow_phase");
      setGameflowPhase(phase);
      
      // 如果启用自动接受且处于ReadyCheck阶段，自动接受匹配
      if (autoAcceptEnabled && phase === "ReadyCheck") {
        await acceptMatch();
      }
    } catch (err) {
      console.error("获取游戏状态失败:", err);
    }
  };

  // 手动接受匹配
  const acceptMatch = async () => {
    try {
      const result: string = await invoke("accept_match");
      setStatus(result);
      setError("");
    } catch (err) {
      setError(`接受匹配失败: ${err}`);
    }
  };

  // 定期检查状态
  useEffect(() => {
    const interval = setInterval(async () => {
      const connected = await checkLcuConnection();
      if (connected) {
        await fetchGameflowPhase();
        if (!summonerInfo) {
          await fetchSummonerInfo();
        }
      }
    }, 2000);

    return () => clearInterval(interval);
  }, [autoAcceptEnabled, summonerInfo]);

  // 初始检查
  useEffect(() => {
    checkLcuConnection();
  }, []);

  const getPhaseDisplayName = (phase: string) => {
    const phaseMap: { [key: string]: string } = {
      "None": "无",
      "Lobby": "房间中",
      "Matchmaking": "匹配中",
      "ReadyCheck": "准备检查",
      "ChampSelect": "英雄选择",
      "InProgress": "游戏中",
      "Reconnect": "重连",
      "WaitingForStats": "等待结算",
      "PreEndOfGame": "游戏结束前",
      "EndOfGame": "游戏结束"
    };
    return phaseMap[phase] || phase;
  };

  return (
    <main className="app-container">
      <div className="header">
        <h1>英雄联盟自动接受助手</h1>
        <div className={`connection-status ${lcuConnected ? 'connected' : 'disconnected'}`}>
          <div className="status-indicator"></div>
          <span>{status}</span>
        </div>
      </div>

      {error && (
        <div className="error-message">
          <span>⚠️ {error}</span>
        </div>
      )}

      {lcuConnected && summonerInfo && (
        <div className="summoner-card">
          <div className="summoner-avatar">
            <img 
              src={`https://ddragon.leagueoflegends.com/cdn/13.24.1/img/profileicon/${summonerInfo.profile_icon_id}.png`}
              alt="召唤师头像"
              onError={(e) => {
                (e.target as HTMLImageElement).src = '/tauri.svg';
              }}
            />
          </div>
          <div className="summoner-info">
            <h2>{summonerInfo.display_name}</h2>
            <p>等级: {summonerInfo.summoner_level}</p>
          </div>
        </div>
      )}

      <div className="game-status">
        <h3>游戏状态</h3>
        <div className="status-item">
          <span className="label">当前阶段:</span>
          <span className={`phase ${gameflowPhase.toLowerCase()}`}>
            {getPhaseDisplayName(gameflowPhase) || "未知"}
          </span>
        </div>
      </div>

      <div className="controls">
        <div className="auto-accept-toggle">
          <label className="toggle-switch">
            <input
              type="checkbox"
              checked={autoAcceptEnabled}
              onChange={(e) => setAutoAcceptEnabled(e.target.checked)}
              disabled={!lcuConnected}
            />
            <span className="slider"></span>
          </label>
          <span className="toggle-label">
            自动接受匹配 {autoAcceptEnabled ? '(已启用)' : '(已禁用)'}
          </span>
        </div>

        <button
          className="accept-button"
          onClick={acceptMatch}
          disabled={!lcuConnected || gameflowPhase !== "ReadyCheck"}
        >
          手动接受匹配
        </button>

        <button
          className="refresh-button"
          onClick={() => {
            checkLcuConnection();
            if (lcuConnected) {
              fetchSummonerInfo();
              fetchGameflowPhase();
            }
          }}
        >
          刷新状态
        </button>
      </div>

      <div className="footer">
        <p>请确保英雄联盟客户端正在运行</p>
        <p>自动接受功能仅在准备检查阶段生效</p>
      </div>
    </main>
  );
}

export default App;
