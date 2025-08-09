import { useState, useEffect } from 'react';
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

  // 获取应用状态
  const getAppState = async () => {
    try {
      const state = await invoke<AppState>('get_app_state');
      setAppState(state);
    } catch (err) {
      console.error('获取应用状态失败:', err);
    }
  };

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

    // 监听窗口移动事件，保存位置
    const setupWindowListeners = async () => {
      const currentWindow = getCurrentWindow();
      
      // 监听窗口移动事件
      const unlistenMoved = await currentWindow.listen('tauri://move', async (event) => {
        const position = event.payload as { x: number; y: number };
        try {
          await invoke('save_window_position', { x: position.x, y: position.y });
          console.log('窗口位置已保存:', position);
        } catch (error) {
          console.error('保存窗口位置失败:', error);
        }
      });

      return unlistenMoved;
    };

    // 定期更新状态（降低频率，主要用于同步状态）
    const interval = setInterval(() => {
      getAppState();
    }, 5000);

    let unlistenMoved: (() => void) | null = null;
    setupWindowListeners().then(fn => {
      unlistenMoved = fn;
    });

    return () => {
      clearInterval(interval);
      unlistenGameflow.then(fn => fn());
      unlistenMatchAccepted.then(fn => fn());
      if (unlistenMoved) {
        unlistenMoved();
      }
    };
  }, []);

  return { appState, getAppState };
};