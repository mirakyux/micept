// 游戏阶段显示名称映射
export const getPhaseDisplayName = (phase: string): string => {
  const phaseMap: Record<string, string> = {
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

// 状态指示器样式类名映射
export const getStatusIndicatorClass = (phase: string): string => {
  const statusMap: Record<string, string> = {
    'None': 'status-none',
    'Lobby': 'status-lobby',
    'InProgress': 'status-inprogress',
    'Reconnect': 'status-reconnect',
    'Matchmaking': 'status-matchmaking',
    'ReadyCheck': 'status-readycheck',
    'ChampSelect': 'status-champselect',
    'WaitingForStats': 'status-waiting',
    'PreEndOfGame': 'status-ending',
    'EndOfGame': 'status-ended'
  };
  return statusMap[phase] || 'status-default';
};

// 计算经验进度百分比
export const calculateXpProgress = (xpSinceLastLevel: number, xpUntilNextLevel: number): number => {
  const total = xpSinceLastLevel + xpUntilNextLevel;
  return total > 0 ? (xpSinceLastLevel / total) * 100 : 0;
};