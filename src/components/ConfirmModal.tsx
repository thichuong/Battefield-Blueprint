import React, { useEffect } from 'react';
import { RotateCcw, Home, AlertTriangle, AlertCircle, X, Check } from 'lucide-react';
import { soundEngine } from '../audio/soundEngine';

export type ConfirmVariant = 'danger' | 'warning' | 'info';
export type ConfirmIconType = 'restart' | 'home' | 'alert' | 'info';

export interface ConfirmModalProps {
  isOpen: boolean;
  title: string;
  subtitle?: string;
  message: string;
  detailNote?: string;
  confirmText?: string;
  cancelText?: string;
  variant?: ConfirmVariant;
  iconType?: ConfirmIconType;
  onConfirm: () => void;
  onCancel: () => void;
}

export const ConfirmModal: React.FC<ConfirmModalProps> = ({
  isOpen,
  title,
  subtitle,
  message,
  detailNote,
  confirmText = 'XÁC NHẬN',
  cancelText = 'HỦY BỎ',
  variant = 'warning',
  iconType = 'alert',
  onConfirm,
  onCancel,
}) => {
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.stopPropagation();
        onCancel();
      } else if (e.key === 'Enter') {
        e.stopPropagation();
        onConfirm();
      }
    };

    window.addEventListener('keydown', handleKeyDown, true);
    return () => {
      window.removeEventListener('keydown', handleKeyDown, true);
    };
  }, [isOpen, onCancel, onConfirm]);

  if (!isOpen) return null;

  const getVariantStyles = () => {
    switch (variant) {
      case 'danger':
        return {
          border: 'border-rose-500/80',
          glow: 'shadow-[0_0_40px_rgba(244,63,94,0.3)]',
          badgeBg: 'bg-rose-950/70 border-rose-500/60 text-rose-400',
          titleColor: 'text-rose-400',
          headerBg: 'bg-rose-950/30',
          confirmBtn:
            'bg-rose-600 hover:bg-rose-500 active:bg-rose-700 text-white shadow-[0_0_20px_rgba(244,63,94,0.4)] border border-rose-400/60',
        };
      case 'warning':
        return {
          border: 'border-amber-500/80',
          glow: 'shadow-[0_0_40px_rgba(245,158,11,0.25)]',
          badgeBg: 'bg-amber-950/70 border-amber-500/60 text-amber-400',
          titleColor: 'text-amber-400',
          headerBg: 'bg-amber-950/30',
          confirmBtn:
            'bg-amber-600 hover:bg-amber-500 active:bg-amber-700 text-slate-950 font-black shadow-[0_0_20px_rgba(245,158,11,0.4)] border border-amber-300',
        };
      case 'info':
      default:
        return {
          border: 'border-cyan-500/80',
          glow: 'shadow-[0_0_40px_rgba(6,182,212,0.3)]',
          badgeBg: 'bg-cyan-950/70 border-cyan-500/60 text-cyan-400',
          titleColor: 'text-cyan-400',
          headerBg: 'bg-cyan-950/30',
          confirmBtn:
            'bg-cyan-600 hover:bg-cyan-500 active:bg-cyan-700 text-white shadow-[0_0_20px_rgba(6,182,212,0.4)] border border-cyan-400/60',
        };
    }
  };

  const styles = getVariantStyles();

  const renderIcon = () => {
    const iconClass = 'w-6 h-6';
    switch (iconType) {
      case 'restart':
        return <RotateCcw className={iconClass} />;
      case 'home':
        return <Home className={iconClass} />;
      case 'info':
        return <AlertCircle className={iconClass} />;
      case 'alert':
      default:
        return <AlertTriangle className={iconClass} />;
    }
  };

  return (
    <div className="fixed inset-0 z-[70] flex items-center justify-center bg-black/85 backdrop-blur-md p-4 animate-in fade-in duration-150">
      <div
        className={`bg-[#030d1a] border-2 ${styles.border} ${styles.glow} rounded-xl w-full max-w-md shadow-2xl flex flex-col font-mono text-slate-200 overflow-hidden relative`}
      >
        {/* Top Decorative Scanning Bar */}
        <div className="h-1 w-full bg-gradient-to-r from-transparent via-cyan-400 to-transparent opacity-80" />

        {/* Header */}
        <div className={`flex items-center justify-between px-5 py-3.5 border-b border-slate-800 ${styles.headerBg}`}>
          <div className="flex items-center gap-3">
            <div className={`p-2 rounded-lg border ${styles.badgeBg} flex items-center justify-center`}>
              {renderIcon()}
            </div>
            <div>
              <h3 className={`text-base font-bold tracking-wider ${styles.titleColor}`}>{title}</h3>
              {subtitle && <p className="text-[13px] text-slate-300 tracking-widest uppercase">{subtitle}</p>}
            </div>
          </div>
          <button
            onClick={() => {
              soundEngine.playSound('click');
              onCancel();
            }}
            className="p-1 hover:bg-slate-800/80 rounded text-slate-400 hover:text-white transition-colors cursor-pointer"
            aria-label="Đóng"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Body Content */}
        <div className="p-5 flex flex-col gap-4 text-sm">
          <div className="text-slate-100 font-semibold leading-relaxed text-sm bg-slate-900/60 p-3.5 rounded-lg border border-slate-800/90">
            {message}
          </div>

          {detailNote && (
            <div className="flex items-start gap-2.5 text-[13px] text-slate-300 bg-slate-950/70 p-3 rounded-lg border border-slate-800/60 leading-relaxed">
              <AlertCircle className="w-4 h-4 text-amber-400 shrink-0 mt-0.5" />
              <span>{detailNote}</span>
            </div>
          )}

          <div className="text-[13px] text-slate-400 font-mono flex items-center justify-between pt-1">
            <span>[Enter]: Đồng ý</span>
            <span>[Esc]: Hủy bỏ</span>
          </div>
        </div>

        {/* Footer Actions */}
        <div className="px-5 py-3.5 border-t border-slate-800/90 bg-slate-950 flex items-center justify-end gap-3">
          <button
            onClick={() => {
              soundEngine.playSound('click');
              onCancel();
            }}
            className="px-4 py-2 bg-slate-900 hover:bg-slate-800 active:scale-95 border border-slate-700 text-slate-200 hover:text-white rounded-lg text-sm font-bold transition-all cursor-pointer"
          >
            {cancelText}
          </button>
          <button
            onClick={() => {
              soundEngine.playSound('order');
              onConfirm();
            }}
            className={`flex items-center gap-1.5 px-4 py-2 rounded-lg text-sm font-bold transition-all cursor-pointer active:scale-95 ${styles.confirmBtn}`}
          >
            <Check className="w-4 h-4" />
            {confirmText}
          </button>
        </div>
      </div>
    </div>
  );
};
