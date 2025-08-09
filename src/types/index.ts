// 应用状态接口定义
export interface SummonerInfo {
  display_name: string;
  summoner_level: number;
  profile_icon_id: number;
  xp_since_last_level: number;
  xp_until_next_level: number;
}

export interface AppState {
  mouse_through: boolean;
  auto_accept: boolean;
  gameflow_phase: string;
  lcu_connected: boolean;
  summoner_info?: SummonerInfo;
}

// 游戏流程阶段类型
export type GameflowPhase = 
  | 'None'
  | 'Lobby'
  | 'Matchmaking'
  | 'ReadyCheck'
  | 'ChampSelect'
  | 'InProgress'
  | 'Reconnect'
  | 'WaitingForStats'
  | 'PreEndOfGame'
  | 'EndOfGame';

// 事件载荷类型
export interface GameflowChangedEvent {
  payload: string;
}

export interface MatchAcceptedEvent {
  payload: any;
}

export interface WindowMoveEvent {
  payload: { x: number; y: number };
}