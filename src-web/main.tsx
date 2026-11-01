import React from 'react';
import ReactDOM from 'react-dom/client';
import '@fontsource-variable/dm-sans';
import '@fontsource-variable/manrope';
import '@fontsource/ibm-plex-mono/400.css';
import './styles.css';
import App from './App';
class ErrorBoundary extends React.Component<{ children: React.ReactNode }, { error: boolean }> {
  state = { error: false };
  static getDerivedStateFromError() {
    return { error: true };
  }
  render() {
    if (this.state.error)
      return (
        <div className="fatal-error">
          <img src="/favicon.svg" alt="" />
          <h1>Let’s reload your workspace.</h1>
          <p>The interface hit an unexpected error. Your saved materials are safe on the server.</p>
          <button className="button primary" onClick={() => location.reload()}>
            Reload studio
          </button>
        </div>
      );
    return this.props.children;
  }
}
ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  </React.StrictMode>,
);
