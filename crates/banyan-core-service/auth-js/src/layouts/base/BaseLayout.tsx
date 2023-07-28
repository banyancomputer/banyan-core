import styles from './BaseLayout.module.css';
export interface IBaseLayout {}
// @ts-ignore
const BaseLayout: React.FC<IBaseLayout> = ({ children }) => {
	return <main className={styles.main}>{children}</main>;
};
export default BaseLayout;
