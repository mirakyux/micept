import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { AppState } from '../types';

export const useAppState = () => {
  const [appState, setAppState] = useState<AppState>({
    mouse_through: true,
    auto_accept: true,
    gameflow_phase: 'None',
    lcu_connected: false,
    summoner_info: undefined
  });

  const intervalRef = useRef<NodeJS.Timeout | null>(null);
  const isActiveRef = useRef(true);

  // 获取应用状态 - 使用useCallback避免重复创建
  const getAppState = useCallback(async () => {
    if (!isActiveRef.current) return;
    
    try {
      const state = await invoke<AppState>('get_app_state');
      setAppState(prevState => {
        // 只有状态真正改变时才更新，减少不必要的重渲染
        if (JSON.stringify(prevState) !== JSON.stringify(state)) {
          return state;
        }
        return prevState;
      });
    } catch (err) {
      console.error('获取应用状态失败:', err);
    }
  }, []);

  // 防抖的窗口位置保存函数
  const saveWindowPositionDebounced = useCallback(
    (() => {
      let timeoutId: NodeJS.Timeout;
      return (position: { x: number; y: number }) => {
        clearTimeout(timeoutId);
        timeoutId = setTimeout(async () => {
          try {
            await invoke('save_window_position', { x: position.x, y: position.y });
          } catch (error) {
            console.error('保存窗口位置失败:', error);
          }
        }, 500); // 500ms防抖
      };
    })(),
    []
  );

  useEffect(() => {
    isActiveRef.current = true;
    
    // 获取初始状态
    getAppState();

    // 监听后台状态变化事件
    const unlistenGameflow = listen('gameflow-changed', (event) => {
      if (!isActiveRef.current) return;
      
      setAppState(prev => {
        const newPhase = event.payload as string;
        if (prev.gameflow_phase !== newPhase) {
          return { ...prev, gameflow_phase: newPhase };
        }
        return prev;
      });
    });

    const unlistenMatchAccepted = listen('match-accepted', (event) => {
      if (!isActiveRef.current) return;
      console.log('匹配已自动接受:', event.payload);
    });

    // 监听LCU连接状态变化
    const unlistenLcuStatus = listen('lcu-status-changed', (event) => {
      if (!isActiveRef.current) return;
      
      setAppState(prev => {
        const connected = event.payload as boolean;
        if (prev.lcu_connected !== connected) {
          return { ...prev, lcu_connected: connected };
        }
        return prev;
      });
    });

    // 监听召唤师信息更新
    const unlistenSummonerInfo = listen('summoner-info-updated', (event) => {
      if (!isActiveRef.current) return;
      
      setAppState(prev => ({
        ...prev,
        summoner_info: event.payload as any
      }));
    });

    // 监听窗口移动事件，保存位置
    const setupWindowListeners = async () => {
      const currentWindow = getCurrentWindow();
      
      // 监听窗口移动事件 - 使用防抖优化
      const unlistenMoved = await currentWindow.listen('tauri://move', (event) => {
        const position = event.payload as { x: number; y: number };
        saveWindowPositionDebounced(position);
      });

      return unlistenMoved;
    };

    // 减少轮询频率，主要依赖事件驱动
    // 只在必要时进行状态同步（比如应用重新获得焦点时）
    const handleVisibilityChange = () => {
      if (!document.hidden && isActiveRef.current) {
        getAppState();
      }
    };

    document.addEventListener('visibilitychange', handleVisibilityChange);

    // 降低轮询频率到30秒，主要作为备用同步机制
    intervalRef.current = setInterval(() => {
      if (isActiveRef.current && !document.hidden) {
        getAppState();
      }
    }, 30000);

    let unlistenMoved: (() => void) | null = null;
    setupWindowListeners().then(fn => {
      unlistenMoved = fn;
    });

    return () => {
      isActiveRef.current = false;
      
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
      
      document.removeEventListener('visibilitychange', handleVisibilityChange);
      
      unlistenGameflow.then(fn => fn());
      unlistenMatchAccepted.then(fn => fn());
      unlistenLcuStatus.then(fn => fn());
      unlistenSummonerInfo.then(fn => fn());
      
      if (unlistenMoved) {
        unlistenMoved();
      }
    };
  }, [getAppState, saveWindowPositionDebounced]);

  return { appState, getAppState };
};