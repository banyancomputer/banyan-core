import { useEffect, useState } from 'react';
import { useTomb } from '@/contexts/tomb';

export interface ITombBucket {
	bucket_id: string;
}

const TombBucket: React.FC<ITombBucket> = ({ bucket_id }) => {
	const { tombInitialized, loadBucket, unlockBucket, lsBucket } = useTomb();
	const [bucketInitialized, setBucketInitialized] = useState<boolean>(false);
	// TODO: Respond to path changes
	const [path, setPath] = useState<string>('/');
	const [ls, setLs] = useState<any[]>([]);

	// Initialize the tombFs
	useEffect(() => {
		const initBucket = async () => {
			try {
				await loadBucket(bucket_id);
				await unlockBucket(bucket_id);
				setBucketInitialized(true);
			} catch (err) {
				console.error(err);
			}
		};
		if (tombInitialized) {
			initBucket();
		}
	}, [tombInitialized, bucket_id]);

	// Mount the bucket on init -- for now just get where we are
	useEffect(() => {
		const mountRoot = async () => {
			try {
				const contentsObj = await lsBucket(bucket_id, path);
				// Convert the array of objects to an array of JSON strings
				const contents = contentsObj.map((item: any) => JSON.stringify(item));
				setLs(contents);
			} catch (err) {
				console.error(err);
			}
		};
		if (bucketInitialized) {
			mountRoot();
		}
	}, [bucketInitialized, path]);

	return (
		<div>
			{bucketInitialized ? (
				<p> Tomb bucket initialized </p>
			) : (
				<p> Something went wrong with tomb-wasm </p>
			)}
			<div>
				{ls ? (
					ls.map((item, index) => <div key={index}>{item}</div>)
				) : (
					<p> Loading... </p>
				)}
			</div>
		</div>
	);
};
export default TombBucket;
