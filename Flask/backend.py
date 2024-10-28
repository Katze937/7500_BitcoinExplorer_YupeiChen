from flask import Flask, render_template, jsonify
import mysql.connector

app = Flask(__name__)

@app.route('/')
def index():
    return render_template('index.html')

@app.route('/latest_blocks')
def latest_blocks():
    # Connect to MySQL
    conn = mysql.connector.connect(
        user='root',
        password='Aisa937!',  # Ensure your password is secure
        host='127.0.0.1',
        database='bitcoin_data'
    )
    cursor = conn.cursor()

    # Fetch the latest 10 blocks with additional metrics
    cursor.execute("""
        SELECT 
            block_height, 
            timestamp, 
            transaction_count, 
            block_size, 
            fees_collected, 
            miner_address, 
            avg_transaction_size 
        FROM blocks 
        ORDER BY id DESC 
        LIMIT 10
    """)
    blocks = cursor.fetchall()

    cursor.close()
    conn.close()

    # Return as JSON
    return jsonify(blocks)

if __name__ == '__main__':
    app.run(debug=True)
